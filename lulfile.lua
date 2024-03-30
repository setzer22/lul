-- @task
function package()
    sh("cargo build --release")
    local date = sh_out("date +%Y-%m-%d--%H-%M-%S").out
    local build_filename = `builds/lul-{date}.tar.gz`
    sh("mkdir -p builds")
    sh(`tar -czf {build_filename} target/release/lul`)
    print(`Package created at {build_filename}`)
end