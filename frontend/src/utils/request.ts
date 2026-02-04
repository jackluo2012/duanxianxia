/**
 * Axios HTTP客户端配置
 * 统一的API请求工具
 */

import axios, { AxiosInstance } from 'axios';
import { config } from '../config';

// 创建axios实例
export const axiosInstance: AxiosInstance = axios.create({
  baseURL: config.apiBaseUrl,
  timeout: config.requestTimeout,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器 - 自动添加Token
axiosInstance.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token && config.headers) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// 响应拦截器 - 统一错误处理
axiosInstance.interceptors.response.use(
  (response) => response.data,
  (error) => {
    if (error.response) {
      // 服务器返回错误状态码
      const { status, data } = error.response;

      switch (status) {
        case 400:
          console.error('[Request] Bad Request:', data.message);
          break;
        case 401:
          console.error('[Request] Unauthorized:', data.message);
          // Token刷新拦截器会处理401
          break;
        case 403:
          console.error('[Request] Forbidden:', data.message);
          break;
        case 404:
          console.error('[Request] Not Found:', data.message);
          break;
        case 500:
          console.error('[Request] Server Error:', data.message);
          break;
        default:
          console.error('[Request] Error:', data.message || error.message);
      }

      return Promise.reject({
        ...error,
        response: {
          ...error.response,
          data: data || { message: error.message },
        },
      });
    } else if (error.request) {
      // 请求已发送但没有收到响应
      console.error('[Request] No Response:', error.message);
      return Promise.reject(error);
    } else {
      // 请求配置出错
      console.error('[Request] Request Error:', error.message);
      return Promise.reject(error);
    }
  }
);

// 便捷的请求方法
export const request = {
  get: <T = any>(url: string, params?: any) =>
    axiosInstance.get<any, T>(url, { params }),

  post: <T = any>(url: string, data?: any) =>
    axiosInstance.post<any, T>(url, data),

  put: <T = any>(url: string, data?: any) =>
    axiosInstance.put<any, T>(url, data),

  delete: <T = any>(url: string, params?: any) =>
    axiosInstance.delete<any, T>(url, { params }),

  patch: <T = any>(url: string, data?: any) =>
    axiosInstance.patch<any, T>(url, data),
};

export default axiosInstance;
