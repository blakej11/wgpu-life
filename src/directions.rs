// For keeping the usage of a pair of objects consistent.

#[derive(Clone, Copy, Debug)]
pub enum RenderDir {
    Forward,
    Reverse,
}

impl RenderDir {
    pub fn dir(frame_num: usize) -> RenderDir {
        if frame_num % 2 == 0 {
            RenderDir::Forward
        } else {
            RenderDir::Reverse
        }
    }
}

// This lets us use two objects together as a (src, dst) pair.
pub struct RenderSources<T> {
    pair: [T; 2],
}

impl<T> RenderSources<T> {
    pub fn new<F>(generate: F) -> Self where
        F: Fn(RenderDir) -> T
    {
        RenderSources {
            pair: [
                 generate(RenderDir::Forward),
                 generate(RenderDir::Reverse),
            ]
        }
    }

    pub fn src(
        &self,
        dir: RenderDir
    ) -> &T {
        match dir {
            RenderDir::Forward => &self.pair[0],
            RenderDir::Reverse => &self.pair[1],
        }
    }

    pub fn dst(
        &self,
        dir: RenderDir
    ) -> &T {
        match dir {
            RenderDir::Forward => &self.pair[1],
            RenderDir::Reverse => &self.pair[0],
        }
    }
}

// This is for when we have a pair of objects but only want to use
// one of them at a time -- one for the forward direction and the
// other for the reverse direction.
pub struct RenderMotion<T> {
    pair: [T; 2],
}

impl<T> RenderMotion<T> {
    pub fn new<F>(generate: F) -> Self where
        F: Fn(RenderDir) -> T
    {
        RenderMotion {
            pair: [
                 generate(RenderDir::Forward),
                 generate(RenderDir::Reverse),
            ]
        }
    }

    pub fn get(
        &self,
        dir: RenderDir
    ) -> &T {
        match dir {
            RenderDir::Forward => &self.pair[0],
            RenderDir::Reverse => &self.pair[1],
        }
    }
}