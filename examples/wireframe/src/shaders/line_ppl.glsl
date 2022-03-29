#version 460
#if COMPUTE_SHADER

struct LineVert{
    vec4 pos;
    vec4 color;
    vec4 width;
};

struct LineMeshVert{
    vec4 pos;
    vec4 color;
};

layout(set = 0, binding = 0) buffer InIdxs{
    uint in_idxs[];
};

layout(set = 0, binding = 1) buffer InVerts{
    LineVert in_verts[];
};

layout(set = 1, binding = 0) buffer OutIdxs{
    uint out_idxs[];
};

layout(set = 1, binding = 1) buffer OutVerts{
    LineMeshVert out_verts[];
};

// width.x: width of the drawing destianation
// width.y: height of the drawing destianation.
// width.w width in pixels.
layout(set = 2, binding = 0) uniform Width{
    vec4 width;
};

layout(push_constant) uniform Camera{
    mat4 pvm;
};

#define v0 in_verts[i0]
#define v1 in_verts[i1]

void main(){
    uint i = gl_GlobalInvocationID.x;

    uint i0 = in_idxs[i * 2];
    uint i1 = in_idxs[i * 2 +1];
    //LineMeshVert v0 = in_verts[i0];
    //LineMeshVert v1 = in_verts[i1];

    uint i00 = i * 4 + 0;
    uint i01 = i * 4 + 1;
    uint i10 = i * 4 + 2;
    uint i11 = i * 4 + 3;

    vec4 v0_pos = pvm * vec4(v0.pos.xyz, 1.);
    vec4 v1_pos = pvm * vec4(v1.pos.xyz, 1.);

    // Transform into device normalized coordinates manually.
    v0_pos /= v0_pos.w;
    v1_pos /= v1_pos.w;

    vec4 off = vec4(normalize(cross(vec3(0, 0, 1), v1_pos.xyz - v0_pos.xyz)), 0) * width.w / vec4(width.xy, 1., 1.);

    out_verts[i00].pos = v0_pos + off * v0.width.x;
    out_verts[i01].pos = v0_pos - off * v0.width.x;
    out_verts[i10].pos = v1_pos + off * v1.width.x;
    out_verts[i11].pos = v1_pos - off * v1.width.x;

    out_verts[i00].color = v0.color;
    out_verts[i01].color = v0.color;
    out_verts[i10].color = v1.color;
    out_verts[i11].color = v1.color;

    uint tr0 = i * 6 + 0;
    uint tr1 = i * 6 + 3;

    out_idxs[tr0 + 0] = i00;
    out_idxs[tr0 + 1] = i01;
    out_idxs[tr0 + 2] = i11;
    out_idxs[tr1 + 0] = i00;
    out_idxs[tr1 + 1] = i11;
    out_idxs[tr1 + 2] = i10;
}

#endif
