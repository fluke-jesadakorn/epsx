
import { Client } from 'pg';

const walletAddress = '0x9Dd4Db1aA7826A94E479f3387A464772f1E2C4B7';
const normalizedAddress = walletAddress.toLowerCase();

// Use fallback config URL or environment variable
const dbUrl = process.env.DATABASE_URL || 'postgresql://epsx_user:password@localhost:5432/epsx_prod'; // Assuming localhost port logic from config

async function checkPermissions() {
    const client = new Client({
        connectionString: dbUrl,
    });

    try {
        await client.connect();
        console.log(`Checking permissions for wallet: ${normalizedAddress}`);

        // 1. Check if user exists
        const userRes = await client.query('SELECT * FROM wallet_users WHERE lower(wallet_address) = $1', [normalizedAddress]);
        if (userRes.rows.length === 0) {
            console.log('❌ User not found in wallet_users table');
            return;
        }
        console.log('✅ User found:', userRes.rows[0]);

        // 2. Check direct permissions
        const directPermsRes = await client.query(`
      SELECT p.permission_string 
      FROM wallet_direct_permissions wdp
      JOIN permissions p ON wdp.permission_id = p.id
      WHERE lower(wdp.wallet_address) = $1 AND wdp.is_active = true
    `, [normalizedAddress]);

        console.log('Direct Permissions:', directPermsRes.rows.map(r => r.permission_string));

        // 3. Check group assignments
        const groupsRes = await client.query(`
      SELECT g.name, g.slug 
      FROM wallet_group_assignments wga
      JOIN groups g ON wga.group_id = g.id
      WHERE lower(wga.wallet_address) = $1 AND wga.is_active = true
    `, [normalizedAddress]);

        console.log('Active Groups:', groupsRes.rows);

        // 4. Check group permissions
        const groupPermsRes = await client.query(`
      SELECT p.permission_string, g.name as group_name
      FROM wallet_group_assignments wga
      JOIN group_permissions gp ON wga.group_id = gp.group_id
      JOIN permissions p ON gp.permission_id = p.id
      JOIN groups g ON wga.group_id = g.id
      WHERE lower(wga.wallet_address) = $1 AND wga.is_active = true
    `, [normalizedAddress]);

        console.log('Group Permissions:', groupPermsRes.rows.map(r => `${r.group_name}: ${r.permission_string}`));

        // 5. Check all distinct permissions strings (simulating what the service does)
        const allPermsRes = await client.query(`
      SELECT DISTINCT p.permission_string
      FROM (
        SELECT permission_id FROM wallet_direct_permissions WHERE lower(wallet_address) = $1 AND is_active = true
        UNION
        SELECT gp.permission_id 
        FROM wallet_group_assignments wga
        JOIN group_permissions gp ON wga.group_id = gp.group_id
        WHERE lower(wga.wallet_address) = $1 AND wga.is_active = true
      ) as user_perms
      JOIN permissions p ON user_perms.permission_id = p.id
    `, [normalizedAddress]);

        const permissions = allPermsRes.rows.map(r => r.permission_string);
        console.log('\n--- Effective Permissions ---');
        console.log(permissions);

        // Analyze for ranking config
        let minOffset = 100;
        let maxLimit = -1;

        for (const perm of permissions) {
            if (perm.startsWith('epsx:rankings:offset:')) {
                const val = parseInt(perm.split(':')[3]);
                if (!isNaN(val) && val < minOffset) minOffset = val;
            }
            if (perm.startsWith('epsx:rankings:limit:') || perm.startsWith('epsx:rankings:view:') || perm.startsWith('epsx:analytics:view:')) {
                const valStr = perm.split(':')[3];
                if (valStr === 'unlimited' || valStr === '-1') {
                    maxLimit = -1;
                    minOffset = 1;
                } else {
                    const val = parseInt(valStr);
                    if (!isNaN(val)) {
                        if (maxLimit !== -1 && val > maxLimit) maxLimit = val;
                        if (val > 10) minOffset = 1;
                    }
                }
            }
            if (perm === 'epsx:*:*' || perm === 'epsx:rankings:*') {
                minOffset = 1;
                maxLimit = -1;
            }
        }
        console.log(`\nCalculated Config: Offset=${minOffset}, Limit=${maxLimit}`);

    } catch (err) {
        console.error('Error querying database:', err);
    } finally {
        await client.end();
    }
}

checkPermissions();
