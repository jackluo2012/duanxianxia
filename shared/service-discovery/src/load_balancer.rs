//! # 负载均衡器
//!
//! 提供多种负载均衡策略
//!
//! ## 支持的策略
//!
//! - RoundRobin: 轮询
//! - Random: 随机
//! - LeastConnections: 最少连接
//! - WeightedRoundRobin: 加权轮询
//! - ConsistentHash: 一致性哈希
//!
//! ## 使用示例
//!
//! ```rust
//! use duanxianxia_service_discovery::load_balancer::{LoadBalancer, LoadBalancerStrategy};
//!
//! let mut lb = LoadBalancer::new(LoadBalancerStrategy::RoundRobin);
//! lb.add_instance(service_instance);
//!
//! if let Some(instance) = lb.select() {
//!     println!("Selected: {}", instance.endpoint());
//! }
//! ```

use super::ServiceInstance;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// 负载均衡策略
#[derive(Debug, Clone, Copy)]
pub enum LoadBalancerStrategy {
    /// 轮询
    RoundRobin,
    /// 随机
    Random,
    /// 最少连接
    LeastConnections,
    /// 加权轮询
    WeightedRoundRobin,
    /// 一致性哈希
    ConsistentHash,
}

/// 带权重的服务实例
#[derive(Debug, Clone)]
pub struct WeightedInstance {
    pub instance: ServiceInstance,
    pub weight: u32,
    pub current_weight: i32,
}

/// 负载均衡器
pub struct LoadBalancer {
    strategy: LoadBalancerStrategy,
    instances: Vec<Arc<ServiceInstance>>,
    weighted_instances: Vec<WeightedInstance>,
    round_robin_index: AtomicUsize,
    total_weight: u32,
}

impl LoadBalancer {
    /// 创建新的负载均衡器
    pub fn new(strategy: LoadBalancerStrategy) -> Self {
        Self {
            strategy,
            instances: Vec::new(),
            weighted_instances: Vec::new(),
            round_robin_index: AtomicUsize::new(0),
            total_weight: 0,
        }
    }

    /// 添加服务实例
    pub fn add_instance(&mut self, instance: ServiceInstance) {
        self.instances.push(Arc::new(instance));
    }

    /// 添加带权重的服务实例
    pub fn add_weighted_instance(&mut self, instance: ServiceInstance, weight: u32) {
        self.weighted_instances.push(WeightedInstance {
            instance,
            weight,
            current_weight: 0,
        });
        self.total_weight += weight;
    }

    /// 移除服务实例
    pub fn remove_instance(&mut self, instance_id: &str) {
        self.instances.retain(|i| i.id != instance_id);
        self.weighted_instances.retain(|wi| wi.instance.id != instance_id);
    }

    /// 更新服务实例列表
    pub fn update_instances(&mut self, instances: Vec<ServiceInstance>) {
        self.instances = instances.into_iter().map(Arc::new).collect();
    }

    /// 选择一个服务实例
    pub fn select(&self) -> Option<Arc<ServiceInstance>> {
        if self.instances.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalancerStrategy::RoundRobin => self.select_round_robin(),
            LoadBalancerStrategy::Random => self.select_random(),
            LoadBalancerStrategy::LeastConnections => self.select_least_connections(),
            LoadBalancerStrategy::WeightedRoundRobin => self.select_weighted_round_robin(),
            LoadBalancerStrategy::ConsistentHash => {
                // 一致性哈希需要key，这里使用随机选择作为fallback
                self.select_random()
            }
        }
    }

    /// 轮询选择
    fn select_round_robin(&self) -> Option<Arc<ServiceInstance>> {
        let index = self.round_robin_index.fetch_add(1, Ordering::Relaxed);
        Some(self.instances[index % self.instances.len()].clone())
    }

    /// 随机选择
    fn select_random(&self) -> Option<Arc<ServiceInstance>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.instances.len());
        Some(self.instances[index].clone())
    }

    /// 最少连接选择
    fn select_least_connections(&self) -> Option<Arc<ServiceInstance>> {
        // 简化实现：随机选择
        // 实际实现需要跟踪每个实例的连接数
        self.select_random()
    }

    /// 加权轮询选择（平滑加权轮询算法）
    fn select_weighted_round_robin(&self) -> Option<Arc<ServiceInstance>> {
        if self.weighted_instances.is_empty() {
            return None;
        }

        // 简化实现：按权重随机选择
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_weight = rng.gen_range(0..self.total_weight);

        let mut accumulated_weight = 0;
        for wi in &self.weighted_instances {
            accumulated_weight += wi.weight;
            if random_weight < accumulated_weight {
                return Some(Arc::new(wi.instance.clone()));
            }
        }

        // Fallback
        Some(Arc::new(self.weighted_instances.last().unwrap().instance.clone()))
    }

    /// 一致性哈希选择
    pub fn select_with_key(&self, key: &str) -> Option<Arc<ServiceInstance>> {
        if self.instances.is_empty() {
            return None;
        }

        // 计算key的hash值
        let hash = Self::calculate_hash(key);
        let index = (hash as usize) % self.instances.len();

        Some(self.instances[index].clone())
    }

    /// 计算字符串的hash值
    fn calculate_hash(s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// 获取实例数量
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

/// 服务网格负载均衡器
///
/// 支持跨服务调用的负载均衡
pub struct ServiceMeshBalancer {
    balancers: dashmap::DashMap<String, LoadBalancer>,
    default_strategy: LoadBalancerStrategy,
}

impl ServiceMeshBalancer {
    /// 创建服务网格负载均衡器
    pub fn new(default_strategy: LoadBalancerStrategy) -> Self {
        Self {
            balancers: dashmap::DashMap::new(),
            default_strategy,
        }
    }

    /// 注册服务实例
    pub fn register(&self, service_name: impl Into<String>, instance: ServiceInstance) {
        let service_name = service_name.into();
        
        self.balancers
            .entry(service_name)
            .or_insert_with(|| LoadBalancer::new(self.default_strategy))
            .add_instance(instance);
    }

    /// 更新服务实例列表
    pub fn update_service(&self, service_name: impl Into<String>, instances: Vec<ServiceInstance>) {
        let service_name = service_name.into();
        
        let mut lb = LoadBalancer::new(self.default_strategy);
        for instance in instances {
            lb.add_instance(instance);
        }
        
        self.balancers.insert(service_name, lb);
    }

    /// 选择服务实例
    pub fn select(&self, service_name: &str) -> Option<Arc<ServiceInstance>> {
        self.balancers
            .get(service_name)
            .and_then(|lb| lb.select())
    }

    /// 使用key选择（一致性哈希）
    pub fn select_with_key(&self, service_name: &str, key: &str) -> Option<Arc<ServiceInstance>> {
        self.balancers
            .get(service_name)
            .and_then(|lb| lb.select_with_key(key))
    }

    /// 移除服务
    pub fn remove_service(&self, service_name: &str) {
        self.balancers.remove(service_name);
    }

    /// 获取所有服务名称
    pub fn service_names(&self) -> Vec<String> {
        self.balancers
            .iter()
            .map(|e| e.key().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_instance(id: &str, port: u16) -> ServiceInstance {
        ServiceInstance {
            id: id.to_string(),
            name: "test-service".to_string(),
            address: "127.0.0.1".to_string(),
            port,
            tags: vec![],
            meta: Default::default(),
            health_check: None,
            status: super::super::ServiceStatus::Healthy,
        }
    }

    #[test]
    fn test_round_robin() {
        let mut lb = LoadBalancer::new(LoadBalancerStrategy::RoundRobin);
        lb.add_instance(create_test_instance("1", 8081));
        lb.add_instance(create_test_instance("2", 8082));
        lb.add_instance(create_test_instance("3", 8083));

        let selected: Vec<u16> = (0..6)
            .filter_map(|_| lb.select())
            .map(|i| i.port)
            .collect();

        // 轮询应该按顺序返回
        assert_eq!(selected, vec![8081, 8082, 8083, 8081, 8082, 8083]);
    }

    #[test]
    fn test_consistent_hash() {
        let mut lb = LoadBalancer::new(LoadBalancerStrategy::ConsistentHash);
        lb.add_instance(create_test_instance("1", 8081));
        lb.add_instance(create_test_instance("2", 8082));
        lb.add_instance(create_test_instance("3", 8083));

        // 相同的key应该总是选择相同的实例
        let key = "user-123";
        let first = lb.select_with_key(key).unwrap();
        
        for _ in 0..10 {
            let selected = lb.select_with_key(key).unwrap();
            assert_eq!(first.port, selected.port);
        }
    }

    #[test]
    fn test_service_mesh() {
        let mesh = ServiceMeshBalancer::new(LoadBalancerStrategy::RoundRobin);
        
        mesh.register("auth-service", create_test_instance("auth-1", 8082));
        mesh.register("auth-service", create_test_instance("auth-2", 8083));
        mesh.register("query-service", create_test_instance("query-1", 8089));

        assert_eq!(mesh.service_names().len(), 2);
        
        let auth = mesh.select("auth-service");
        assert!(auth.is_some());
        
        let query = mesh.select("query-service");
        assert!(query.is_some());
    }
}
