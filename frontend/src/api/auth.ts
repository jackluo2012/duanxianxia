/**
 * 认证相关 API
 */

import request from './request';
import { config } from '../config';

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  refreshToken: string;
  user: UserInfo;
  expiresIn: number;
}

export interface UserInfo {
  id: string;
  username: string;
  email?: string;
  nickname?: string;
  avatar?: string;
  role?: string;
  createdAt: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
  nickname?: string;
}

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
