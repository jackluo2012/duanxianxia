/**
 * 认证相关类型定义
 */

export interface LoginRequest {
  username: string;
  password: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
  nickname?: string;
}

export interface LoginResponse {
  token: string;
  refreshToken: string;
  user: UserInfo;
  expiresIn: number;
}

/**
 * 用户基本信息
 */
export interface UserInfo {
  id: string;
  username: string;
  email?: string;
  nickname?: string;
  avatar?: string;
  role?: string;
  createdAt: string;
}

/**
 * 角色信息
 */
export interface Role {
  id: string;
  name: string;
  code: string;
  description?: string;
  isSystem: boolean;
  permissions: Permission[];
  createdAt: string;
  updatedAt: string;
}

/**
 * 权限信息
 */
export interface Permission {
  id: string;
  name: string;
  code: string;
  resource: string;
  action: string;
  description?: string;
  module?: string;
  createdAt: string;
  updatedAt: string;
}

/**
 * 用户权限响应
 */
export interface UserPermissionsResponse {
  roles: Role[];
  permissions: Permission[];
  allPermissions: string[]; // 权限代码数组，方便检查
}

/**
 * 订阅计划类型
 */
export type SubscriptionPlan = 'free' | 'basic' | 'premium' | 'enterprise';

/**
 * 订阅信息
 */
export interface Subscription {
  id: string;
  plan: SubscriptionPlan;
  status: 'active' | 'inactive' | 'cancelled' | 'expired';
  startDate: string;
  endDate?: string;
  autoRenew: boolean;
  features: string[];
}

/**
 * 用户完整信息（包含订阅和权限）
 */
export interface UserProfile extends UserInfo {
  roles?: Role[];
  permissions?: Permission[];
  subscription?: Subscription;
}

/**
 * 分配角色请求
 */
export interface AssignRoleRequest {
  userId: string;
  roleIds: string[];
}

/**
 * 分配角色响应
 */
export interface AssignRoleResponse {
  success: boolean;
  message: string;
}