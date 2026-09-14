'use strict';
let state, provider, prepared, busy = false;
const el = id => document.getElementById(id);
const status = text => { el('status').textContent = text; };
const providers = [];
window.addEventListener('eip6963:announceProvider', event => {
  if (event.detail.info.rdns === 'io.metamask') providers.push(event.detail.provider);
});
window.dispatchEvent(new Event('eip6963:requestProvider'));
function wallet() {
  provider ??= providers[0] || (window.ethereum?.isMetaMask ? window.ethereum : undefined);
  if (!provider) throw new Error('MetaMask is unavailable in this browser. Open this page in Chrome with MetaMask.');
  return provider;
}
async function api(path, body) {
  const response = await fetch(path, body ? {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)} : {});
  const value = await response.json();
  if (!response.ok) throw new Error(value.error || 'Request failed');
  return value;
}
async function accountStatus() {
  const accounts = await wallet().request({method:'eth_accounts'});
  const chain = await wallet().request({method:'eth_chainId'});
  el('account').textContent = accounts[0] || 'Not connected';
  el('network').textContent = `${parseInt(chain,16) === 31337 ? 'local Anvil' : 'Switch network required'} (${parseInt(chain,16)})`;
  return {account:accounts[0],chain};
}
async function requireAccount() {
  const current = await accountStatus();
  if (parseInt(current.chain,16) !== 31337) throw new Error('Select local Anvil (31337) in MetaMask.');
  if (current.account?.toLowerCase() !== state.admin.toLowerCase()) throw new Error(`Select the configured Admin wallet ${state.admin} in MetaMask.`);
  return current;
}
async function refresh() {
  state = await api('/state');
  el('admin').textContent = state.admin;
  el('contracts').replaceChildren();
  for (const name of state.contracts) {
    const row = document.createElement('div'); row.className = 'contract';
    const title = document.createElement('strong'); title.textContent = name; row.append(title);
    const record = state.records[name];
    const text = document.createElement('span'); text.textContent = record ? `${record.status} · ${record.address || record.transactionHash}` : 'Ready to prepare'; row.append(text, document.createElement('br'));
    const button = document.createElement('button'); button.textContent = record ? `Verify ${name}` : `Prepare ${name}`;
    button.onclick = () => action(async () => {
      if (record) { const result = await api('/verify',{name}); status(`${name}: ${result.status}`); await refresh(); return; }
      await requireAccount();
      status(`Estimating gas for ${name} on local Anvil…`);
      prepared = await api('/prepare',{name});
      el('transaction').textContent = JSON.stringify({contract:name,network:'local Anvil (31337)',from:prepared.transaction.from,value:'0 tBNB',maximumGasCostTBNB:prepared.maximumGasCostTBNB,gasLimit:parseInt(prepared.transaction.gas,16),creationSha256:prepared.creationSha256,constructor:name.startsWith('Mock') ? 'No arguments; 18 decimals test token' : {admin:state.admin,treasury:state.treasury,tokens:[state.records.MockUSDT?.address,state.records.MockUSDC?.address]}},null,2);
      el('review').hidden = false; status('Transaction prepared. Review the details before requesting MetaMask confirmation.');
    });
    row.append(button); el('contracts').append(row);
  }
  if (provider) await accountStatus();
}
async function action(fn) {
  if (busy) return;
  busy = true;
  document.querySelectorAll('button').forEach(b=>b.disabled=true);
  try { await fn(); } catch(error) { status(error.message || String(error)); }
  finally { busy=false; document.querySelectorAll('button').forEach(b=>b.disabled=false); }
}
el('connect').onclick = () => action(async () => {
  status('Choose the configured Admin account in the MetaMask connection prompt.');
  await wallet().request({method:'eth_requestAccounts'}); await accountStatus(); status('Connected. Select local Anvil and the configured Admin wallet.');
  provider.on('accountsChanged',()=>{prepared=null;el('review').hidden=true;accountStatus().catch(e=>status(e.message));});
  provider.on('chainChanged',()=>{prepared=null;el('review').hidden=true;accountStatus().catch(e=>status(e.message));});
});
el('switch').onclick = () => action(async () => {
  try { await wallet().request({method:'wallet_switchEthereumChain',params:[{chainId:'0x7a69'}]}); }
  catch(error) {
    if (error.code !== 4902) throw error;
    await wallet().request({method:'wallet_addEthereumChain',params:[{chainId:'0x7a69',chainName:'EPSX Local',nativeCurrency:{name:'Test BNB',symbol:'tBNB',decimals:18},rpcUrls:[state.rpc]}]});
  }
  await accountStatus(); status('Network status refreshed.');
});
el('refresh').onclick = () => action(refresh);
el('send').onclick = () => action(async () => {
  if (!prepared) throw new Error('Prepare a transaction first.');
  await requireAccount();
  const plan = prepared; prepared=null; el('review').hidden=true;
  status(`Confirm ${plan.name} deployment in MetaMask on local Anvil (31337).`);
  const hash = await wallet().request({method:'eth_sendTransaction',params:[plan.transaction]});
  localStorage.setItem(`epsx-local-${plan.name}`,hash);
  status(`Submitted ${plan.name}: ${hash}. Keep this hash until verification succeeds.`);
  const result = await api('/submitted',{name:plan.name,hash});
  status(`${plan.name}: ${result.status} · ${hash}`); await refresh();
});
action(async()=>{await refresh();status('Pinned build loaded. Connect MetaMask to begin.');});
