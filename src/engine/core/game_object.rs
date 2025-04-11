impl GameObject {
    pub fn update(&mut self, transform: Transform) {
        // Update transform
        self.transform = transform;
        
        // Update components
        if let Some(model) = &mut self.model {
            model.update(&self.transform);
        }
    }
} 