use rapier2d::prelude::*;

pub struct PhysicsState {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub center_handle: RigidBodyHandle,
    pub outer_handles: Vec<RigidBodyHandle>,
    pub splat_active: bool,
    pub center_to_outer_joints: Vec<ImpulseJointHandle>,
}

impl PhysicsState {
    pub fn new() -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();
        let mut impulse_joint_set = ImpulseJointSet::new();

        // center node
        let center_rb = RigidBodyBuilder::dynamic().translation(vector![100.0, 100.0].into()).linear_damping(5.0).build();
        let center_handle = rigid_body_set.insert(center_rb);
        
        let mut outer_handles = Vec::new();
        let num_nodes = 64;
        let radius = 60.0;
        
        let mut center_to_outer_joints = Vec::new();

        for i in 0..num_nodes {
            let angle = (i as f32) * std::f32::consts::TAU / (num_nodes as f32);
            let x = 100.0 + radius * angle.cos();
            let y = 100.0 + radius * angle.sin();
            
            let outer_rb = RigidBodyBuilder::dynamic().translation(vector![x, y].into()).linear_damping(2.0).build();
            let handle = rigid_body_set.insert(outer_rb);
            outer_handles.push(handle);
            
            // Connect to center
            let joint = SpringJointBuilder::new(radius, 200.0, 10.0).local_anchor1(point![0.0, 0.0].into()).local_anchor2(point![0.0, 0.0].into());
            let j_handle = impulse_joint_set.insert(center_handle, handle, joint, true);
            center_to_outer_joints.push(j_handle);
        }
        
        // Connect outer nodes to each other
        for i in 0..num_nodes {
            let h1 = outer_handles[i];
            let h2 = outer_handles[(i + 1) % num_nodes];
            let dist = (std::f32::consts::TAU / num_nodes as f32).sin() * radius;
            let joint = SpringJointBuilder::new(dist, 1000.0, 20.0).local_anchor1(point![0.0, 0.0].into()).local_anchor2(point![0.0, 0.0].into());
            impulse_joint_set.insert(h1, h2, joint, true);
        }

        Self {
            rigid_body_set,
            collider_set,
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set,
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            center_handle,
            outer_handles,
            splat_active: false,
            center_to_outer_joints,
        }
    }

    pub fn step(&mut self) {
        let gravity = vector![0.0, 0.0];
        let physics_hooks = ();
        let event_handler = ();
        self.physics_pipeline.step(
            vector![0.0, 0.0].into(),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &physics_hooks,
            &event_handler,
        );
    }
}
