import axios, { AxiosInstance } from 'axios';

// 创建一个已解包响应的Axios实例类型
type UnwrappedAxiosInstance = Omit<AxiosInstance, 'get' | 'post' | 'put' | 'delete' | 'patch'> & {
  get: <T>(url: string, config?: any) => Promise<T>;
  post: <T>(url: string, data?: any, config?: any) => Promise<T>;
  put: <T>(url: string, data?: any, config?: any) => Promise<T>;
  delete: <T>(url: string, config?: any) => Promise<T>;
  patch: <T>(url: string, data?: any, config?: any) => Promise<T>;
};

const request = axios.create({
  baseURL: '/api',
  timeout: 10000,
}) as UnwrappedAxiosInstance;

// 请求拦截器 - 自动携带 JWT
request.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 响应拦截器 - 统一错误处理
request.interceptors.response.use(
  (response) => response.data,
  (error) => {
    if (error.response?.status === 401) {
      // 跳转登录页
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

export default request;
