pub trait Scene {
    fn hold();
}

pub struct Scenes;

impl Scenes {
    pub fn together(scenes: &[fn()]) {
        std::thread::scope(|scope| {
            let started: Vec<_> = scenes.iter().map(|scene| scope.spawn(scene)).collect();
            started.into_iter().try_for_each(std::thread::ScopedJoinHandle::join).unwrap();
        });
    }
}
