/**
 * 认证相关 API
 */

import request from './request';
import type {
  LoginRequest,
  LoginResponse,
  UserInfo,
  RegisterRequest,
  Role,
  Permission,
  UserPermissionsResponse,
  AssignRoleRequest,
  AssignRoleResponse,
} from '../types/auth';

/**
 * 用户登录
 * @param credentials 登录凭证
 * @returns 登录响应 (token + user info)
 */
export async function login(credentials: LoginRequest): Promise<LoginResponse> {
  return request.post('/auth/login', credentials);
}

/**
 * 用户注册
 * @param data 注册信息
 * @returns 注册响应
 */
export async function register(data: RegisterRequest): Promise<LoginResponse> {
  return request.post('/auth/register', data);
}

/**
 * 刷新Token
 * @param refreshToken 刷新令牌
 * @returns 新的登录响应
 */
export async function refreshToken(refreshToken: string): Promise<LoginResponse> {
  return request.post('/auth/refresh', { refreshToken });
}

/**
 * 获取当前用户信息
 * @returns 用户信息
 */
export async function getCurrentUser(): Promise<UserInfo> {
  return request.get('/auth/me');
}

/**
 * 用户登出
 * @returns 登出结果
 */
export async function logout(): Promise<void> {
  return request.post('/auth/logout');
}

// ==================== RBAC 相关 API ====================

/**
 * 获取所有角色列表
 * @returns 角色列表
 */
export async function getRoles(): Promise<Role[]> {
  return request.get('/auth/roles');
}

/**
 * 获取所有权限列表
 * @returns 权限列表
 */
export async function getPermissions(): Promise<Permission[]> {
  return request.get('/auth/permissions');
}

/**
 * 获取当前用户的权限和角色
 * @returns 用户权限和角色信息
 */
export async function getUserPermissions(): Promise<UserPermissionsResponse> {
  return request.get('/auth/me/permissions');
}

/**
 * 为用户分配角色
 * @param data 分配角色请求
 * @returns 分配结果
 */
export async function assignRoleToUser(data: AssignRoleRequest): Promise<AssignRoleResponse> {
  return request.post('/auth/users/roles', data);
}

/**
 * 获取指定用户的权限信息
 * @param userId 用户ID
 * @returns 用户权限和角色信息
 */
export async function getUserPermissionsById(userId: string): Promise<UserPermissionsResponse> {
  return request.get(`/auth/users/${userId}/permissions`);
}

// 重新导出类型，保持向后兼容
export type {
  LoginRequest,
  LoginResponse,
  UserInfo,
  RegisterRequest,
  Role,
  Permission,
  UserPermissionsResponse,
  AssignRoleRequest,
  AssignRoleResponse,
};
