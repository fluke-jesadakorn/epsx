#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a3-12-indexer-schema-boundary.sh"
contract="$repo_root/docs/migration/contracts/a3-12-indexer-schema-boundary.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-12-indexer-schema.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "indexer runtime DDL 5→0; three guarded public tables, 27 exact columns and chain-scoped transaction PK pinned" "$temp_dir/integrity.out"
grep -q "31 exact constraints and 10 exact btree indexes fail closed" "$temp_dir/integrity.out"
grep -q "autonomous provider, placeholder sync and fabricated ingestion are absent" "$temp_dir/integrity.out"
grep -q "all four surviving runtime relations are public-qualified; only health remains reachable" "$temp_dir/integrity.out"
grep -q "dormant fork store pins eight guarded tables, 74 columns, 101 constraints and two explicit indexes without runtime activation" "$temp_dir/integrity.out"
grep -q "ten-name fresh-create preflight rejects every public relation-kind collision before CREATE" "$temp_dir/integrity.out"
grep -q "nine ordered dormant-adapter source byte/SHA-256 pins are recomputed before semantic anchors" "$temp_dir/integrity.out"
grep -q "private journal and atomic apply callsite pin replay before lease without journal transaction ownership or runtime activation" "$temp_dir/integrity.out"
grep -q "default-off private PostgreSQL substrate pins port delegation, candidates, codecs, leases, reads, journal and apply without runtime activation" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
[ "$readiness_status" -eq 3 ] || { cat "$temp_dir/readiness.out" >&2; exit 1; }
grep -q "ten residual A3.12 blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const r = await Bun.file(process.argv[1]).json();
if (r.productionReady !== false || r.readinessExit !== 3) process.exit(1);
if (r.provenance.standaloneSourceIndexer !== false) process.exit(1);
if (r.runtimeRust.files !== 14 || r.runtimeRust.ddlFindings !== 0 || r.runtimeRust.expectedDelta !== -5 || r.runtimeRust.fakeSyncAvailable !== false) process.exit(1);
if (Object.values(r.runtimeRust.qualifiedRelations).reduce((a,b) => a+b, 0) !== 4) process.exit(1);
if (r.dormantAdapter.status !== "compiled-static-substrate" || r.dormantAdapter.feature !== "dormant-postgres-adapter" || r.dormantAdapter.defaultEnabled !== false || r.dormantAdapter.privateModule !== true || r.dormantAdapter.publicExport !== false || r.dormantAdapter.mainCallsite !== false || r.dormantAdapter.poolHolderOnly !== false) process.exit(1);
if (!Array.isArray(r.dormantAdapter.sourcePins) || r.dormantAdapter.sourcePins.length !== 9 || r.dormantAdapter.sourcePins.some((pin) => typeof pin.path !== "string" || !Number.isInteger(pin.bytes) || !/^[0-9a-f]{64}$/.test(pin.sha256))) process.exit(1);
if (r.dormantAdapter.parentConflictTargetOnly !== true || r.dormantAdapter.strictChildInserts !== true || r.dormantAdapter.fullCandidateReload !== true || r.dormantAdapter.reloadRevalidation !== true || r.dormantAdapter.databaseClockLeasePredicates !== true || r.dormantAdapter.persistentLeaseFence !== true) process.exit(1);
if (r.dormantAdapter.readSide.helpers !== true || r.dormantAdapter.readSide.modulePrivate !== true || r.dormantAdapter.readSide.repeatableReadOnlyTransactions !== true || r.dormantAdapter.readSide.candidateReadSingleSnapshot !== true || r.dormantAdapter.readSide.snapshotReadSingleSnapshot !== true) process.exit(1);
if (r.dormantAdapter.readSide.absentStateRejectsSelectedOrJournalOrphans !== true || r.dormantAdapter.readSide.snapshotMappingChecks !== true || r.dormantAdapter.readSide.snapshotRevisionChecks !== true || r.dormantAdapter.readSide.selectedHashLeftJoinsChainState !== true || r.dormantAdapter.readSide.selectedHashRejectsMissingStaleOrFutureState !== true || r.dormantAdapter.readSide.candidateReadsLegacyProjection !== false) process.exit(1);
if (r.dormantAdapter.journal.helpers !== true || r.dormantAdapter.journal.modulePrivate !== true || r.dormantAdapter.journal.replayIfPresentPrimitive !== true || r.dormantAdapter.journal.replayMustPrecedeLease !== true || r.dormantAdapter.journal.replayBeforeLeaseCallsite !== true || r.dormantAdapter.journal.applyCoordinator !== true || r.dormantAdapter.journal.traitImplementation !== true) process.exit(1);
if (r.dormantAdapter.journal.exactHeaderReplay !== true || r.dormantAdapter.journal.orderedRoleLocalDenseMembers !== true || r.dormantAdapter.journal.fullAttachedCandidateReload !== true || r.dormantAdapter.journal.malformedStoredMapsCorruptState !== true || r.dormantAdapter.journal.requestOrContentDriftMapsMutationIdReuse !== true) process.exit(1);
if (r.dormantAdapter.journal.appendAppliedMutationInsertOnly !== true || r.dormantAdapter.journal.mutationDerivedOutcomeAndRevision !== true || r.dormantAdapter.journal.fingerprint !== false || r.dormantAdapter.journal.updateOrDelete !== false || r.dormantAdapter.journal.transactionOwnership !== false || r.dormantAdapter.journal.activated !== false) process.exit(1);
if (r.dormantAdapter.tests.defaultLibraryPassed !== 33 || r.dormantAdapter.tests.featureLibraryPassed !== 62 || r.dormantAdapter.tests.binaryPassed !== 4) process.exit(1);
if (r.dormantAdapter.atomicityDatabaseProof !== false || r.dormantAdapter.concurrencyDatabaseProof !== false || r.dormantAdapter.cancellationDatabaseProof !== false) process.exit(1);
if (r.dormantAdapter.databaseRead !== false || r.dormantAdapter.databaseWrite !== false || r.dormantAdapter.migrationExecuted !== false || r.dormantAdapter.runtimeAdapter !== false || r.dormantAdapter.providerActivated !== false || r.dormantAdapter.workerActivated !== false || r.dormantAdapter.routeActivated !== false || r.dormantAdapter.executed !== false) process.exit(1);
if (r.migrationRoot.migrations !== 2 || r.migrationRoot.projection.pinnedBytes !== 4822 || r.migrationRoot.projection.guardedTables !== 3 || r.migrationRoot.projection.guardedIndexes !== 5) process.exit(1);
if (r.migrationRoot.forkStore.pinnedBytes !== 23326 || r.migrationRoot.forkStore.guardedTables !== 8 || r.migrationRoot.forkStore.guardedIndexes !== 2) process.exit(1);
if (r.schema.tables !== 3 || r.schema.columns !== 27 || r.schema.structuralConstraints !== 7 || r.schema.checkConstraints !== 24 || r.schema.indexes !== 10) process.exit(1);
if (JSON.stringify(r.schema.transactionPrimaryKey) !== JSON.stringify(["chain_id", "hash"])) process.exit(1);
if (r.forkStore.status !== "dormant-static-substrate" || r.forkStore.tables !== 8 || r.forkStore.columns !== 74 || r.forkStore.structuralConstraints !== 28 || r.forkStore.checkConstraints !== 73 || r.forkStore.explicitIndexes !== 2 || r.forkStore.collisionPreflight !== true || r.forkStore.collisionNames !== 10 || r.forkStore.freshCreateOnly !== true || r.forkStore.runtimeProbe !== false || r.forkStore.executed !== false) process.exit(1);
if (r.blockers.length !== 10) process.exit(1);
' "$temp_dir/report-one.json"

tamper() {
  name=$1
  expression=$2
  expected=$3
  out="$temp_dir/$name.json"
  A3_IN="$contract" A3_OUT="$out" A3_EXPR="$expression" bun -e '
    const value = await Bun.file(process.env.A3_IN).json();
    Function("value", process.env.A3_EXPR)(value);
    await Bun.write(process.env.A3_OUT, `${JSON.stringify(value, null, 2)}\n`);
  '
  set +e
  "$verify" --mode integrity --contract "$out" >"$temp_dir/$name.out" 2>&1
  status=$?
  set -e
  [ "$status" -eq 1 ] || { cat "$temp_dir/$name.out" >&2; echo "tamper $name unexpectedly passed" >&2; exit 1; }
  grep -Eq "$expected" "$temp_dir/$name.out" || { cat "$temp_dir/$name.out" >&2; exit 1; }
}

tamper production-ready 'value.productionReady = true' 'readiness sentinel changed'
tamper source-commit 'value.provenance.sourceCommit = "0".repeat(40)' 'development provenance drifted'
tamper source-path 'value.provenance.absentPaths[0] = "apps/backend"' 'absent development paths drifted'
tamper runtime-blob 'value.provenance.removedRuntimeSnapshot.blob = "0".repeat(40)' 'removed runtime snapshot pin drifted'
tamper query-digest 'value.runtimeBoundary.compatibilityQuerySha256 = "0".repeat(64)' 'compatibility query bytes changed'
tamper query-bytes 'value.runtimeBoundary.compatibilityQueryBytes -= 1' 'compatibility query bytes changed'
tamper structural-array-type 'value.runtimeBoundary.structuralKeyArrayTextCastOccurrences = 1' 'structural key-array type contract drifted'
tamper relation-count 'value.runtimeBoundary.qualifiedRelationOccurrences["public.blocks"] = 1' 'public.blocks runtime occurrence count'
tamper rust-inventory 'value.runtimeBoundary.rustInventory.pop()' 'Rust inventory drifted'
tamper adapter-source-pin-hash 'value.dormantAdapterBoundary.sourcePins[2].sha256 = "0".repeat(64)' 'dormant adapter source pins drifted'
tamper adapter-source-pin-bytes 'value.dormantAdapterBoundary.sourcePins[3].bytes -= 1' 'dormant adapter source pins drifted'
tamper adapter-source-pin-path 'value.dormantAdapterBoundary.sourcePins[4].path = "services/indexer/src/main.rs"' 'dormant adapter source pins drifted'
tamper adapter-source-pin-order 'value.dormantAdapterBoundary.sourcePins.reverse()' 'dormant adapter source pins drifted'
tamper adapter-journal-source-pin 'value.dormantAdapterBoundary.sourcePins[5].sha256 = "e".repeat(64)' 'dormant adapter source pins drifted'
tamper adapter-read-source-pin 'value.dormantAdapterBoundary.sourcePins[8].sha256 = "f".repeat(64)' 'dormant adapter source pins drifted'
tamper adapter-feature 'value.dormantAdapterBoundary.feature = "always-on-postgres"' 'dormant adapter boundary drifted'
tamper adapter-read-module 'value.dormantAdapterBoundary.readModulePrivate = false' 'dormant adapter boundary drifted'
tamper adapter-consistent-read 'value.dormantAdapterBoundary.repeatableReadOnlyTransactions = false' 'dormant adapter boundary drifted'
tamper adapter-read-orphan 'value.dormantAdapterBoundary.absentStateRejectsSelectedOrJournalOrphans = false' 'dormant adapter boundary drifted'
tamper adapter-read-future 'value.dormantAdapterBoundary.selectedHashRejectsMissingStaleOrFutureState = false' 'dormant adapter boundary drifted'
tamper adapter-read-legacy 'value.dormantAdapterBoundary.candidateReadsLegacyProjection = true' 'dormant adapter boundary drifted'
tamper adapter-read-activation 'value.dormantAdapterBoundary.runtimeAdapter = true' 'dormant adapter boundary drifted'
tamper adapter-journal-privacy 'value.dormantAdapterBoundary.journalModulePrivate = false' 'dormant adapter boundary drifted'
tamper adapter-journal-replay 'value.dormantAdapterBoundary.replayIfPresentPrimitive = false' 'dormant adapter boundary drifted'
tamper adapter-journal-callsite 'value.dormantAdapterBoundary.replayBeforeLeaseCallsite = false' 'dormant adapter boundary drifted'
tamper adapter-apply-coordinator 'value.dormantAdapterBoundary.applyCoordinator = false' 'dormant adapter boundary drifted'
tamper adapter-trait-implementation 'value.dormantAdapterBoundary.journalTraitImplementation = false' 'dormant adapter boundary drifted'
tamper adapter-atomicity-proof 'value.dormantAdapterBoundary.atomicityDatabaseProof = true' 'dormant adapter boundary drifted'
tamper adapter-journal-full-candidate 'value.dormantAdapterBoundary.fullAttachedCandidateReload = false' 'dormant adapter boundary drifted'
tamper adapter-journal-insert-only 'value.dormantAdapterBoundary.appendAppliedMutationInsertOnly = false' 'dormant adapter boundary drifted'
tamper adapter-journal-fingerprint 'value.dormantAdapterBoundary.journalFingerprint = true' 'dormant adapter boundary drifted'
tamper adapter-journal-transaction-owner 'value.dormantAdapterBoundary.journalTransactionOwnership = true' 'dormant adapter boundary drifted'
tamper adapter-journal-activation 'value.dormantAdapterBoundary.journalActivated = true' 'dormant adapter boundary drifted'
tamper adapter-module-privacy 'value.dormantAdapterBoundary.modulePrivate = false' 'dormant adapter boundary drifted'
tamper adapter-db-clock 'value.dormantAdapterBoundary.databaseClockLeasePredicates = false' 'dormant adapter boundary drifted'
tamper adapter-strict-child 'value.dormantAdapterBoundary.strictChildInserts = false' 'dormant adapter boundary drifted'
tamper adapter-activation 'value.dormantAdapterBoundary.routeActivated = true' 'dormant adapter boundary drifted'
tamper fake-sync-policy 'value.runtimeBoundary.forbiddenRuntimeAnchors.pop()' 'unsafe runtime anchor returned|runtime boundary|drifted'
tamper migration-hash 'value.migrationRoot.orderedMigrations[0].sha256 = "0".repeat(64)' 'ordered migration pin drifted'
tamper migration-guard 'value.migrationRoot.orderedMigrations[0].guards.pop()' 'migration root boundary drifted|migration guard|drifted'
tamper fork-migration-hash 'value.migrationRoot.orderedMigrations[1].sha256 = "0".repeat(64)' 'fork-store migration pin drifted'
tamper fork-migration-bytes 'value.migrationRoot.orderedMigrations[1].bytes -= 1' 'fork-store migration pin drifted'
tamper fork-migration-guard 'value.migrationRoot.orderedMigrations[1].guards.pop()' 'fork-store migration guards drifted'
tamper fork-table-inventory 'delete value.forkStoreContract.tables.indexer_mutation_blocks' 'fork-store contract descriptors drifted'
tamper fork-preflight-required 'value.forkStoreContract.freshCreateCollisionPreflight = false' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-preflight-order 'value.forkStoreContract.preflightBeforeCreates = false' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-preflight-relkind 'value.forkStoreContract.preflightRelkindRestricted = true' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-preflight-name 'value.forkStoreContract.collisionPreflightNames.pop()' 'fork-store contract descriptors drifted|fork-store collision contract names drifted'
tamper fork-if-not-exists-policy 'value.forkStoreContract.ifNotExistsAloneSafe = true' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-baseline-adoption 'value.forkStoreContract.baselineAdoption = true' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-version-order 'value.forkStoreContract.futureRunnerRecordsVersionAfterPreflight = false' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-procedural-begin 'value.forkStoreContract.proceduralBeginOnlyInCollisionPreflight = false' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-transaction-control 'value.forkStoreContract.topLevelTransactionControl = true' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-forbidden-policy 'value.migrationRoot.forbiddenTokens.push("BEGIN")' 'migration forbidden token policy drifted'
tamper fork-height-key 'value.forkStoreContract.candidateHeightPrimaryKey = true' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-global-tx-key 'value.forkStoreContract.globalTransactionHashUnique = true' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-selected-fk 'value.forkStoreContract.selectedCandidateTripleForeignKey = false' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-lease-pair 'value.forkStoreContract.pairedLeaseOwnerExpiry = false' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-fact-flags 'value.forkStoreContract.factCanonicalOrFinalizedFlags = true' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-payload-caps 'value.forkStoreContract.fixedPayloadCaps = true' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper fork-fingerprint 'value.forkStoreContract.mutationFingerprint = true' 'fork-store contract descriptors drifted|fork-store static policy drifted'
tamper global-hash-pk 'value.schemaContract.globalTransactionHashPrimaryKeyAccepted = true' 'schema fail-closed policy drifted'
tamper structural-fk 'value.schemaContract.structuralConstraints.pop()' 'schema structural constraint descriptors drifted'
tamper column-substitution 'value.schemaContract.tables.blocks[0] = "chain_ix:varchar(10):required"' 'schema column descriptors drifted'
tamper structural-substitution 'value.schemaContract.structuralConstraints[0] = "blocks_pkey:pk(chain_ix,number)"' 'schema structural constraint descriptors drifted'
tamper weak-check-substitution 'value.schemaContract.checkConstraints[1] = "blocks_number_check:blocks:check(number<=0)"' 'schema check constraint descriptors drifted'
tamper pg18-status-shape 'value.schemaContract.checkConstraints[15] = "transactions_status_check:transactions:check(statusisnullorstatus=any(array[0,1]))"' 'schema check constraint descriptors drifted'
tamper index-substitution 'value.schemaContract.indexes[0] = "blocks_pkey:blocks:unique-primary:btree(chain_ix,number)"' 'schema index descriptors drifted'
tamper pg18-index-quote 'value.schemaContract.indexes[2] = value.schemaContract.indexes[2].replace("\"timestamp\"", "timestampxx")' 'schema index descriptors drifted'
tamper check-count 'value.schemaContract.checkConstraints.pop()' 'schema check constraint descriptors drifted'
tamper index-inventory 'value.schemaContract.indexes.pop()' 'schema index descriptors drifted'
tamper inheritance 'value.schemaContract.inheritanceAccepted = true' 'schema fail-closed policy drifted'
tamper rls 'value.schemaContract.rowLevelSecurityAccepted = true' 'schema fail-closed policy drifted'
tamper opclass 'value.schemaContract.nonCatalogOperatorClassesAccepted = true' 'schema fail-closed policy drifted'
tamper collation 'value.schemaContract.indexCollationDriftAccepted = true' 'schema fail-closed policy drifted'
tamper partial-index 'value.schemaContract.partialExpressionOrIncludedIndexesAccepted = true' 'schema fail-closed policy drifted'
tamper blockers 'value.blockers.pop()' 'exact ten residual blockers are required'

echo "a3-12-indexer-schema-boundary self-test: PASS"
