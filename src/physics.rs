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
        let mut collider_set = ColliderSet::new();
        let mut impulse_joint_set = ImpulseJointSet::new();

        // center node
        let center_rb = RigidBodyBuilder::dynamic().translation(vector![100.0, 100.0].into()).additional_mass(1.0).linear_damping(1.0).build();
        let center_handle = rigid_body_set.insert(center_rb);
        let center_col = ColliderBuilder::ball(40.0).build();
        collider_set.insert_with_parent(center_col, center_handle, &mut rigid_body_set);
        
        let mut outer_handles = Vec::new();
        let num_nodes = 64;
        let radius = 60.0;
        
        let mut center_to_outer_joints = Vec::new();

        for i in 0..num_nodes {
            let angle = (i as f32) * std::f32::consts::TAU / (num_nodes as f32);
            let x = 100.0 + radius * angle.cos();
            let y = 100.0 + radius * angle.sin();
            
            let outer_rb = RigidBodyBuilder::dynamic().translation(vector![x, y].into()).additional_mass(0.1).linear_damping(0.5).build();
            let handle = rigid_body_set.insert(outer_rb);
            let col = ColliderBuilder::ball(2.5).build();
            collider_set.insert_with_parent(col, handle, &mut rigid_body_set);
            outer_handles.push(handle);
            
            // Connect to center
            let joint = SpringJointBuilder::new(radius, 50.0, 2.0).local_anchor1(point![0.0, 0.0].into()).local_anchor2(point![0.0, 0.0].into());
            let j_handle = impulse_joint_set.insert(center_handle, handle, joint, true);
            center_to_outer_joints.push(j_handle);
        }
        
        // Connect outer nodes to each other
        for i in 0..num_nodes {
            let h1 = outer_handles[i];
            let h2 = outer_handles[(i + 1) % num_nodes];
            // exact chord length
            let dist = 2.0 * radius * (std::f32::consts::PI / num_nodes as f32).sin();
            let joint = SpringJointBuilder::new(dist, 100.0, 5.0).local_anchor1(point![0.0, 0.0].into()).local_anchor2(point![0.0, 0.0].into());
            impulse_joint_set.insert(h1, h2, joint, true);
        }

        // Wall colliders
        let thickness = 100.0;
        let screen_w = 1920.0;
        let screen_h = 1080.0;
        let top = ColliderBuilder::cuboid(screen_w, thickness).translation(vector![screen_w / 2.0, -thickness].into()).build();
        collider_set.insert(top);
        let bottom = ColliderBuilder::cuboid(screen_w, thickness).translation(vector![screen_w / 2.0, screen_h + thickness].into()).build();
        collider_set.insert(bottom);
        let left = ColliderBuilder::cuboid(thickness, screen_h).translation(vector![-thickness, screen_h / 2.0].into()).build();
        collider_set.insert(left);
        let right = ColliderBuilder::cuboid(thickness, screen_h).translation(vector![screen_w + thickness, screen_h / 2.0].into()).build();
        collider_set.insert(right);

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
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
            use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
            use windows::Win32::Foundation::POINT;

            unsafe {
                let state = GetAsyncKeyState(VK_LBUTTON.0 as i32);
                if (state as u16 & 0x8000) != 0 {
                    let mut pt = POINT { x: 0, y: 0 };
                    if GetCursorPos(&mut pt).is_ok() {
                        if let Some(center_rb) = self.rigid_body_set.get_mut(self.center_handle) {
                            let pos = center_rb.translation();
                            let dx = pt.x as f32 - pos.x;
                            let dy = pt.y as f32 - pos.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist < 250.0 { // Grab radius
                                let force = vector![dx * 2000.0, dy * 2000.0];
                                center_rb.apply_impulse(force.into(), true);
                            }
                        }
                    }
                }
            }
        }

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
