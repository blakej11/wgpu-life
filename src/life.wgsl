struct LifeParams {
    width : u32,
    height : u32,
    threshold : f32,
}

struct Cells {
    cells : array<f32>,
}

@group(0) @binding(0) var<uniform> params : LifeParams;
@group(0) @binding(1) var<storage> cellSrc :  Cells;
// probably we can change it to write only, but we need to change description
@group(0) @binding(2) var<storage, read_write> cellDst :  Cells;
@group(0) @binding(3) var texture :  texture_storage_2d<r32float, write>;

@compute 
@workgroup_size(8, 8)
fn life(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let X : u32 = global_id.x;
    let Y : u32 = global_id.y;
    let W : u32 = params.width;
    let H : u32 = params.height;
    let thresh : f32 = params.threshold;

    if (X > W || Y > H) {
        return;
    }

    var count : i32 = 0;
    for (var y : i32 = i32(Y - 1u); y <= i32(Y + 1u); y++ ) {
        for (var x : i32 = i32(X - 1u); x <= i32(X + 1u); x = x + 1) {
            let yw : u32 = u32(y + i32(H)) % H;
            let xw : u32 = u32(x + i32(W)) % W;
            if (cellSrc.cells[yw * W + xw] > thresh) {
                count = count + 1;
            }
        }
    }

    let pix : u32 = Y * W + X;
    let ov : f32 = cellSrc.cells[pix];
    let was_alive : bool = ov > thresh;
    var nv : f32;

    // in the first clause, "3 or 4" includes the center cell
    if (was_alive && (count == 3 || count == 4)) {
        if (ov - 0.01 > thresh) {
            nv = ov - 0.01;
        } else {
            nv = ov;
        }
    } else {
        if (!was_alive && count == 3) {
            nv = 1.0;
        } else {
            nv = 0.0;
        }
    }

    cellDst.cells[pix] = nv;

    textureStore(texture,
        vec2<i32>(i32(X), i32(Y)),
        // all channels other than the first are ignored
        vec4<f32>(nv, 0.0, 0.0, 1.0));
}
