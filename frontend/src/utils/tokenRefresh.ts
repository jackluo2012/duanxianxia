/**
 * Token刷新拦截器
 * 在API请求失败时自动刷新Token并重试
 */

import { AxiosInstance } from 'axios';
import { useAuthStore } from '../stores/authStore';

let isRefreshing = false;
let failedRequests: Array<{
  resolve: (value?: any) => void;
  reject: (reason?: any) => void;
}> = [];

export function setupTokenRefreshInterceptor(axiosInstance: AxiosInstance) {
  // 响应拦截器
  axiosInstance.interceptors.response.use(
    (response) => response,
    async (error) => {
      const originalRequest = error.config;

      // 如果是401错误且不是刷新Token的请求
      if (
        error.response?.status === 401 &&
        !originalRequest?._retry &&
        originalRequest?.url !== '/auth/refresh'
      ) {
        if (isRefreshing) {
          // 如果正在刷新，将请求加入队列
          return new Promise((resolve, reject) => {
            failedRequests.push({ resolve, reject });
          }).then((token) => {
            originalRequest.headers.Authorization = `Bearer ${token}`;
            return axiosInstance(originalRequest);
          });
        }

        isRefreshing = true;

        try {
          // 刷新Token
          const authStore = useAuthStore.getState();
          const success = await authStore.refresh();

          if (success) {
            // 刷新成功，重试所有失败的请求
            failedRequests.forEach(({ resolve }) => {
              resolve(authStore.token);
            });

            // 更新当前请求的token
            originalRequest.headers.Authorization = `Bearer ${authStore.token}`;
            originalRequest._retry = true;

            return axiosInstance(originalRequest);
          } else {
            // 刷新失败，清除队列
            failedRequests.forEach(({ reject }) => {
              reject(error);
            });
            failedRequests = [];

            // 跳转到登录页
            window.location.href = '/login';
          }
        } catch (refreshError) {
          // 刷新异常，清除队列
          failedRequests.forEach(({ reject }) => {
            reject(refreshError);
          });
          failedRequests = [];

          throw error;
        } finally {
          isRefreshing = false;
        }
      }

      return Promise.reject(error);
    }
  );
}
