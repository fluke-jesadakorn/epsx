SELECT kind || ':' || object_name || ':' || definition AS entry FROM (
 SELECT 'column' AS kind, n.nspname || '.' || c.relname || '.' || a.attname AS object_name,
        jsonb_build_array(a.attnum, format_type(a.atttypid,a.atttypmod), a.attnotnull,
          a.attidentity, a.attgenerated, pg_get_expr(d.adbin,d.adrelid))::text AS definition
 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
 JOIN pg_attribute a ON a.attrelid=c.oid
 LEFT JOIN pg_attrdef d ON d.adrelid=c.oid AND d.adnum=a.attnum
 WHERE n.nspname NOT IN ('pg_catalog','information_schema') AND n.nspname NOT LIKE 'pg_toast%' AND c.relkind IN ('r','p','v','m','f') AND a.attnum>0 AND NOT a.attisdropped
   AND c.relname NOT IN ('epsx_schema_migrations','epsx_schema_adoptions')
 UNION ALL
 SELECT 'constraint', n.nspname || '.' || c.relname || '.' || k.conname, pg_get_constraintdef(k.oid,true)
 FROM pg_constraint k JOIN pg_class c ON c.oid=k.conrelid JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname NOT IN ('pg_catalog','information_schema') AND n.nspname NOT LIKE 'pg_toast%' AND c.relname NOT IN ('epsx_schema_migrations','epsx_schema_adoptions')
 UNION ALL
 SELECT 'index', schemaname || '.' || tablename || '.' || indexname, indexdef
 FROM pg_indexes WHERE schemaname NOT IN ('pg_catalog','information_schema') AND schemaname NOT LIKE 'pg_toast%' AND tablename NOT IN ('epsx_schema_migrations','epsx_schema_adoptions')
 UNION ALL
 SELECT 'function', n.nspname || '.' || p.proname || '(' || pg_get_function_identity_arguments(p.oid) || ')', pg_get_functiondef(p.oid)
 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE n.nspname NOT IN ('pg_catalog','information_schema') AND n.nspname NOT LIKE 'pg_toast%' AND p.prokind IN ('f','p')
 UNION ALL
 SELECT 'view', n.nspname || '.' || c.relname, pg_get_viewdef(c.oid,true)
 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname NOT IN ('pg_catalog','information_schema') AND n.nspname NOT LIKE 'pg_toast%' AND c.relkind IN ('v','m')
 UNION ALL
 SELECT 'trigger', n.nspname || '.' || c.relname || '.' || t.tgname, pg_get_triggerdef(t.oid,true)
 FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname NOT IN ('pg_catalog','information_schema') AND n.nspname NOT LIKE 'pg_toast%' AND NOT t.tgisinternal
 UNION ALL
 SELECT 'enum', n.nspname || '.' || t.typname || '.' || e.enumsortorder::text, e.enumlabel
 FROM pg_type t JOIN pg_enum e ON e.enumtypid=t.oid JOIN pg_namespace n ON n.oid=t.typnamespace WHERE n.nspname NOT IN ('pg_catalog','information_schema') AND n.nspname NOT LIKE 'pg_toast%'
 UNION ALL
 SELECT 'extension', extname, extversion FROM pg_extension
 UNION ALL
 SELECT 'relation', n.nspname || '.' || c.relname, jsonb_build_array(c.relkind,c.relrowsecurity,c.relforcerowsecurity,c.reloptions)::text
 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname NOT IN ('pg_catalog','information_schema') AND n.nspname NOT LIKE 'pg_toast%' AND c.relkind IN ('r','p','v','m','f') AND c.relname NOT IN ('epsx_schema_migrations','epsx_schema_adoptions')
 UNION ALL
 SELECT 'policy', schemaname || '.' || tablename || '.' || policyname, jsonb_build_array(permissive,roles,cmd,qual,with_check)::text
 FROM pg_policies WHERE schemaname NOT IN ('pg_catalog','information_schema') AND schemaname NOT LIKE 'pg_toast%'
) catalog ORDER BY kind COLLATE "C", object_name COLLATE "C", definition COLLATE "C"
