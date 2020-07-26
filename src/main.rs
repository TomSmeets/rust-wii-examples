#![no_std]
#![feature(start)]

extern crate alloc;
use mllib_sys::*;
use ogc::mem_cached_to_uncached;
use ogc::prelude::*;

mod math;
mod util;
use util::*;

static mut screenMode: *mut GXRModeObj = core::ptr::null_mut();
static mut frameBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
static mut readyForCopy: u8 = 0;
const FIFO_SIZE: u32 = 256 * 1024;
const NULL: *mut core::ffi::c_void = core::ptr::null_mut();

#[repr(align(32))]
struct A32;

struct Align<A, T> {
    _alignment: [A; 0],
    value: T,
}

impl<A, T> Align<A, T> {
    const fn new(t: T) -> Self {
        Align {
            _alignment: [],
            value: t,
        }
    }
}

#[rustfmt::skip]
static mut vertices: Align<A32, [i16; 3 * 3]> = Align::new([
      0,  15, 0,
    -15, -15, 0,
     15, -15, 0,
]);

#[rustfmt::skip]
static mut colors: Align<A32, [u8; 3 * 4]> = Align::new([
    255, 0, 0, 255, // red
    0, 255, 0, 255, // green
    0, 0, 255, 255, // blue
]);

pub unsafe fn fail() {
    loop {
        VIDEO_WaitVSync();
    }
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    unsafe {
        let backgroundColor: GXColor = GXColor {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        let mut camera = guVector {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut up = guVector {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };
        let mut look = guVector {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        };

        let mut view: Mtx = core::mem::zeroed();
        let mut projection: Mtx = core::mem::zeroed();

        VIDEO_Init();

        // TODO: add wpad support
        WPAD_Init();

        screenMode = VIDEO_GetPreferredMode(NULL as _);

        frameBuffer = mem_cached_to_uncached!(SYS_AllocateFramebuffer(screenMode));

        VIDEO_Configure(screenMode);
        VIDEO_SetNextFramebuffer(frameBuffer);
        VIDEO_SetPostRetraceCallback(Some(copy_buffers));
        VIDEO_SetBlack(false);
        VIDEO_Flush();

        let fifoBuffer = mem_cached_to_uncached!(memalign(32, FIFO_SIZE as usize));
        memset(fifoBuffer, 0, FIFO_SIZE);

        GX_Init(fifoBuffer, FIFO_SIZE);
        GX_SetCopyClear(backgroundColor, 0x00ffffff);
        GX_SetViewport(
            0.0,
            0.0,
            (*screenMode).fbWidth as f32,
            (*screenMode).efbHeight as f32,
            0.0,
            1.0,
        );
        GX_SetDispCopyYScale((*screenMode).xfbHeight as f32 / (*screenMode).efbHeight as f32);
        GX_SetScissor(
            0,
            0,
            (*screenMode).fbWidth as u32,
            (*screenMode).efbHeight as u32,
        );
        GX_SetDispCopySrc(0, 0, (*screenMode).fbWidth, (*screenMode).efbHeight);
        GX_SetDispCopyDst((*screenMode).fbWidth, (*screenMode).xfbHeight);
        GX_SetCopyFilter(
            (*screenMode).aa,
            (*screenMode).sample_pattern.as_mut_ptr(),
            GX_TRUE as u8,
            (*screenMode).vfilter.as_mut_ptr(),
        );
        GX_SetFieldMode(
            (*screenMode).field_rendering,
            if (*screenMode).viHeight == 2 * (*screenMode).xfbHeight {
                GX_ENABLE as u8
            } else {
                GX_DISABLE as u8
            },
        );

        GX_SetCullMode(GX_CULL_NONE as u8);
        GX_CopyDisp(frameBuffer, GX_TRUE as u8);
        GX_SetDispCopyGamma(GX_GM_1_0 as u8);

        // triangles.as_mut_ptr() as u64;

        let ratio = (*screenMode).fbWidth as f32 / (*screenMode).efbHeight as f32;

        guPerspective(projection.as_mut_ptr(), 60.0, ratio, 10.0, 300.0);
        GX_LoadProjectionMtx(projection.as_mut_ptr(), GX_PERSPECTIVE as u8);

        GX_ClearVtxDesc();
        GX_SetVtxDesc(GX_VA_POS as u8, GX_INDEX8 as u8);
        GX_SetVtxDesc(GX_VA_CLR0 as u8, GX_INDEX8 as u8);
        GX_SetVtxAttrFmt(GX_VTXFMT0 as u8, GX_VA_POS, GX_POS_XYZ, GX_S16, 0);
        GX_SetVtxAttrFmt(GX_VTXFMT0 as u8, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
        GX_SetArray(
            GX_VA_POS,
            vertices.value.as_mut_ptr() as _,
            3 * core::mem::size_of::<i16>() as u8,
        );
        GX_SetArray(
            GX_VA_CLR0,
            colors.value.as_mut_ptr() as _,
            4 * core::mem::size_of::<u8>() as u8,
        );
        GX_SetNumChans(1);
        GX_SetNumTexGens(0);
        GX_SetTevOrder(
            GX_TEVSTAGE0 as u8,
            GX_TEXCOORDNULL as u8,
            GX_TEXMAP_NULL,
            GX_COLOR0A0 as u8,
        );
        GX_SetTevOp(GX_TEVSTAGE0 as u8, GX_PASSCLR as u8);

        loop {
            guLookAt(view.as_mut_ptr(), &mut camera, &mut up, &mut look);
            GX_SetViewport(
                0.0,
                0.0,
                (*screenMode).fbWidth as f32,
                (*screenMode).efbHeight as f32,
                0.0,
                1.0,
            );
            GX_InvVtxCache();
            GX_InvalidateTexAll();
            update_screen(view);

            WPAD_ScanPads();
            if (WPAD_ButtonsDown(0) & WPAD_BUTTON_HOME) != 0 {
                return 0;
            }
        }
    }
}

unsafe fn update_screen(mut viewMatrix: Mtx) {
    let viewMatrix = math::Mtx::from(viewMatrix);
    let mut modelView = math::Mtx::identity().transform(0.0, 0.0, -50.0);
    if (WPAD_ButtonsDown(0) & WPAD_BUTTON_A) != 0 {
        modelView = modelView.transform(0.0, 10.0, 0.0);
    }
    let mut modelView = viewMatrix.concat(modelView);

    GX_LoadPosMtxImm(modelView.inner_mut(), GX_PNMTX0);
    GX_Begin(GX_TRIANGLES as u8, GX_VTXFMT0 as u8, 3);

    // NOTE: Order matters!
    GX_Position1x8(0);
    GX_Color1x8(0);
    GX_Position1x8(1);
    GX_Color1x8(1);
    GX_Position1x8(2);
    GX_Color1x8(2);

    GX_End();

    GX_DrawDone();

    readyForCopy = GX_TRUE as u8;
    VIDEO_WaitVSync();
    return;
}

unsafe extern "C" fn copy_buffers(_count: u32) {
    if readyForCopy == GX_TRUE as u8 {
        GX_SetZMode(GX_TRUE as u8, GX_LEQUAL as u8, GX_TRUE as u8);
        GX_SetColorUpdate(GX_TRUE as u8);
        GX_CopyDisp(frameBuffer, GX_TRUE as u8);
        GX_Flush();
        readyForCopy = GX_FALSE as u8;
    }
}
