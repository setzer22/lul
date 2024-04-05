-- @task
function docker_image()
    sh("docker build -t lul-builder .")
end

function container_id(opts)
    opts = opts or {}

    local running_container = nil
    if opts.new_container then
        print("Killing all previous instances of the container before starting...")
        sh("docker ps -q --filter ancestor=lul-builder | xargs -r docker kill")
        
        print("Starting container...")
        running_container = sh_out(`docker run -v $PWD:/repo -d lul-builder tail -f /dev/null`).out
    else
        running_container = sh_out(`docker ps -q --filter ancestor=lul-builder`).out
    end

    return running_container
end

function build_in_docker(binary_path)
    local container_id = container_id({new_container=true})
    
    sh(`docker exec -it {container_id} bash -c "cd /repo && cargo build --release"`)
    sh(`docker cp {container_id}:/repo/target/release/lul {binary_path}`)
end

-- @task
function package()
    build_in_docker("builds/lul")
    local date = sh_out("date +%Y-%m-%d--%H-%M-%S").out
    local build_filename = `builds/lul-{date}.tar.gz`
    sh("mkdir -p builds")
    sh(`tar -czf {build_filename} builds/lul`)
    print(`Package created at {build_filename}`)
end