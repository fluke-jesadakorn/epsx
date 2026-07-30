function canonicalRequiredPermission(
  permission: string
): [string, string, string] | null {
  const parts = permission.split(':');
  if (parts.length !== 3 || parts.some(part => part === '' || part === '*')) {
    return null;
  }
  return [parts[0], parts[1], parts[2]];
}

export function permissionAllows(
  heldPermissions: readonly string[],
  requiredPermission: string
): boolean {
  const required = canonicalRequiredPermission(requiredPermission);
  if (required === null) {
    return false;
  }
  return heldPermissions.some(permission => {
    if (permission === '*:*' || permission === '*:*:*') {
      return true;
    }
    const parts = permission.split(':');
    if (
      parts.length !== 3 ||
      parts.some(part => part === '') ||
      parts[0] === '*' ||
      parts[0] !== required[0]
    ) {
      return false;
    }
    if (parts[1] === '*' && parts[2] === '*') {
      return true;
    }
    if (parts[1] !== required[1]) {
      return false;
    }
    return parts[2] === '*' || parts[2] === required[2];
  });
}
