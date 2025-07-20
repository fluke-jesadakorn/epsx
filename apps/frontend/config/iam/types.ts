export interface PermissionDocument {
  id: string;
  name: string;
  description: string;
  category: string;
  action: string;
  resource: string;
  scope: string;
  isSystem: boolean;
  tags: string[];
}

export interface RoleDocument {
  id: string;
  name: string;
  description: string;
  color: string;
  icon: string;
  permissions: string[];
  isSystem: boolean;
  isActive: boolean;
}
