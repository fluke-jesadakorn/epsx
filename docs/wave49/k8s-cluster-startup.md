# K8s cluster startup (wave 49 cleanup)

## Issue

The `colima-epsx` K8s cluster context exists in `kubectl config`
but is unreachable:

```
$ kubectl get pods -n epsx-dev
The connection to the server 127.0.0.1:64426 was refused
```

This causes the 4/5 `epsx-dev` pods to show `ImagePullBackOff`
in dashboards. The cluster isn't actually running — the docker
context file for `colima-epsx` is missing on disk.

## Root cause

The `com.epsx.colima-start.plist` LaunchAgent
(`infrastructure/scripts/com.epsx.colima-start.plist`) calls
`/opt/homebrew/bin/colima start --profile epsx` on boot, but
colima is not installed at that path:

```
$ which colima
colima not found
$ ls /opt/homebrew/bin/colima
No such file or directory
$ brew list colima
Error: No such keg: /opt/homebrew/Cellar/colima
```

## Fix

1. Install colima: `brew install colima`
2. Start the cluster: `colima start --profile epsx` (matches
   the kustomize dev overlay's `epsx-dev` namespace + 30100-
   30199 NodePort range from the wave-11 plan)
3. Install the LaunchAgent for auto-start:
   `cp infrastructure/scripts/com.epsx.colima-start.plist ~/Library/LaunchAgents/`
4. Apply the dev overlay:
   `kubectl apply -k infrastructure/kubernetes/overlays/dev`
5. Verify: `kubectl get pods -n epsx-dev` should show 5/5
   Running (the 5 base + the 4 wave-49 pay deployments)

## Status (wave 49 batch B)

- ❌ Not deployed yet (requires colima install + cluster start)
- ✅ Code is ready (all 4 pay K8s manifests + overlays validate)
- ✅ Pay service + BFF smoke-tested locally via the
  `epsx_pay_dev` database
