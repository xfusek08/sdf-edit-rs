struct ShaderInput {
    @builtin(num_workgroups)         num_workgroups: vec3<u32>;
    @builtin(workgroup_id)           workgroup_id: vec3<u32>,
    @builtin(local_invocation_index) local_invocation_index: u32;
    @builtin(local_invocation_id)    local_id: vec3<u32>,
}

// SVO: Definition of SVO in storage memory into which data will be generated
// --------------------------------------------------------------------------

struct SVORoot {
    node_count: atomic<u32>, // number of nodes in tiles buffer, use to atomically add new nodes
    brick_count: atomic<u32> // number of bricks in bricks buffer, use to atomically add new bricks
    max_nodes: u32,          // maximum number of nodes in tiles buffer (expected to be pre-set by host)
    max_bricks: u32,         // maximum number of bricks in bricks buffer (expected to be pre-set by host)
}

@group(0) @binding(0) var<storage, read_write> root: SVORoot;
@group(0) @binding(1) var<storage, read_write> node_headers: array<u32>;
@group(0) @binding(2) var<storage, read_write> node_payload: array<u32>;
@group(0) @binding(3) var<storage, read_write> node_vertices: array<vec4<f32>>;
@group(0) @binding(4) var brick_pool: texture_storage_3d<r32float>;

// JOB buffer in storage memory where unfinished jobs will wait for groups to be taken
// -----------------------------------------------------------------------------------

struct Job {
    status:     u32, // 1: `job` waiting for evaluation, 0: `empty` - new job can be placed here, 2: `locked` - some other thread is writing to this part of job buffer
    node_index: u32, // Which node will be evalutaeds in this job
}
struct JobBufferMeta {
    job_count: atomic<u32>, // when this number is 0, all jobs are finished.
    max_jobs:  u32,         // maximum number of jobs in buffer (expected to be pre-set by host)
}
struct AssignedJob {
    job_index:  u32, // index of job in job buffer
    node_index: u32, // index of node to be evaluted in node buffer, if no jobs are or will be available -1 is retuned.
}
@group(1) @binding(0) var<storage, read_write> job_meta: JobBufferMeta;
@group(1) @binding(1) var<storage, read_write> job_buffer: array<Job>; // expected to be initialized by host to zeros

// Functions
// ---------

var<workgroup> assigned_job: AssignedJob;
fn takeJob(in: ShaderInput) -> AssignedJob {
    // let only one thread from group do job selection
    if (in.local_invocation_index == 0u) {
        loop {
            var job_index = atomicLoad(&job_meta.job_count);
            
            // exit condition - work is finished
            if (job_index == 0u) {
                assigned_job = AssignedJob(-1, -1);
                break;
            }
            
            // try to take job
            if (atomicCompareExchangeWeak(&job_meta.job_count, job_index, job_index - 1u)) {
                // job taken, now find it in job buffer
                for (var i = 0u; i < job_meta.max_jobs; i++) {
                    if (atomicCompareExchangeWeak(&job_buffer[i].status, 1u, 2u)) {
                        assigned_job = AssignedJob(i, job_buffer[i].node_index);
                        break;
                    }
                }
                break;
            }
        }
    }
    workgroupBarrier();
    return assigned_job;
}

@compute
@workgroup_size(8, 8, 8)
fn main(in: ShaderInput) {
    
    // generate first job to evaluate root node
    
    // start job sexecution loop using job buffer
    loop {
        var job: AssignedJob = takeJob();
        if (job.node_index == -1) {
            break;
        }
        
        var node: Node = loadNode(job.node_index);
        var brick_evalutaion_result: BrickEvaluationResult = evaluate_node_brick(node); // As side effect: New brick might be added to brick pool, node_payload is updated, result is same for all threads in workgroup.
        if brick_evalutaion_result == BRICK_IS_BOUONDARY {
            var new_tile_first_node: u32 = subdivide_node(node); // As side effect: New initialized node is added to node pool, node_header is updated to point to new tile index, node index of new tile is returned.
            spawn_jobs_for_tile(new_tile_first_node);
            finish_job(job, 8); // 8 jobs were spawned
        } else {
            finish_job(job, 0); // no jobs were spawned
        }
    }
}
