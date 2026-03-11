#!/bin/bash
# EPSX GitLab Runner Registration Script
# This script registers a local GitLab runner with the project.

RUNNER_TOKEN="glrt-MHyQCz7I6dldAmfOdXEtT286MQpwOjEKdDozCnU6eQ8.01.171dadp75"
GITLAB_URL="https://gitlab.jesadakorn.com/"

echo "Registering GitLab Runner for EPSX..."

gitlab-runner register \
  --non-interactive \
  --url "$GITLAB_URL" \
  --token "$RUNNER_TOKEN" \
  --executor "docker" \
  --docker-image "docker:27" \
  --docker-privileged \
  --docker-volumes "/var/run/docker.sock:/var/run/docker.sock" \
  --docker-network-mode "host"

echo "Runner registered successfully!"
echo "You can now start the runner service with:"
echo "  gitlab-runner run"
echo "Or install it as a system service:"
echo "  gitlab-runner install"
echo "  gitlab-runner start"
