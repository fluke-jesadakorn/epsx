import { describe, expect, test } from 'bun:test';

import { permissionAllows } from './fixture-permissions';

describe('permissionAllows', () => {
  test('accepts only canonical exact and supported wildcard grants', () => {
    const required = 'admin:content:manage';
    for (const permission of [
      required,
      '*:*',
      '*:*:*',
      'admin:*:*',
      'admin:content:*',
    ]) {
      expect(permissionAllows([permission], required)).toBe(true);
    }
  });

  test('rejects partial, cross-namespace, and malformed wildcard grants', () => {
    const required = 'admin:content:manage';
    for (const permission of [
      'admin:*:read',
      '*:content:manage',
      'epsx:*:*',
      'admin:content',
      'admin:content:manage:extra',
      'admin::manage',
    ]) {
      expect(permissionAllows([permission], required)).toBe(false);
    }
  });
});
