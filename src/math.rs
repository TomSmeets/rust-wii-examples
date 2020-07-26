#[derive(Clone, Copy)]
pub struct Mtx {
    inner: ogc_sys::Mtx,
}

impl Mtx {
    pub fn from(inner: ogc_sys::Mtx) -> Self {
        Mtx { inner }
    }

    pub fn zero() -> Self {
        let inner: ogc_sys::Mtx;
        unsafe {
            inner = core::mem::zeroed();
        }
        Mtx { inner }
    }

    pub fn identity() -> Self {
        let mut mtx = Self::zero();
        unsafe {
            ogc_sys::c_guMtxIdentity(mtx.inner.as_mut_ptr());
        }
        mtx
    }

    pub unsafe fn inner_mut(&mut self) -> *mut [f32; 4] {
        self.inner.as_mut_ptr()
    }

    pub unsafe fn inner(&self) -> *const [f32; 4] {
        self.inner.as_ptr()
    }

    pub fn concat(mut self, mut b: Mtx) -> Mtx {
        let mut c = Mtx::zero();
        unsafe {
            ogc_sys::c_guMtxConcat(self.inner_mut(), b.inner_mut(), c.inner_mut());
        }
        c
    }

    pub fn transform(mut self, x: f32, y: f32, z: f32) -> Mtx {
        unsafe {
            ogc_sys::c_guMtxTransApply(self.inner_mut(), self.inner_mut(), x, y, z);
        }
        self
    }

}
