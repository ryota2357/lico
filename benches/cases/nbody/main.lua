local PI = 3.141592653589793
local SOLAR_MASS = 4 * PI * PI
local DAYS_PER_YEAR = 365.24

local Body = {}

function Body.new(x, y, z, vx, vy, vz, mass)
    return setmetatable({
        x = x,
        y = y,
        z = z,
        vx = vx * DAYS_PER_YEAR,
        vy = vy * DAYS_PER_YEAR,
        vz = vz * DAYS_PER_YEAR,
        mass = mass * SOLAR_MASS,
    }, { __index = Body })
end

function Body:offset_momentum(px, py, pz)
    self.vx = -(px / SOLAR_MASS)
    self.vy = -(py / SOLAR_MASS)
    self.vz = -(pz / SOLAR_MASS)
end

function Body.jupiter()
    return Body.new(
        4.84143144246472090,
        -1.16032004402742839,
        -0.103622044471123109,
        0.00166007664274403694,
        0.00769901118419740425,
        -0.0000690460016972063023,
        0.000954791938424326609
    )
end

function Body.saturn()
    return Body.new(
        8.34336671824457987,
        4.12479856412430479,
        -0.403523417114321381,
        -0.00276742510726862411,
        0.00499852801234917238,
        0.0000230417297573763929,
        0.000285885980666130812
    )
end

function Body.uranus()
    return Body.new(
        12.8943695621391310,
        -15.1111514016986312,
        -0.223307578892655734,
        0.00296460137564761618,
        0.00237847173959480950,
        -0.0000296589568540237556,
        0.0000436624404335156298
    )
end

function Body.neptune()
    return Body.new(
        15.3796971148509165,
        -25.9193146099879641,
        0.179258772950371181,
        0.00268067772490389322,
        0.00162824170038242295,
        -0.0000951592254519715870,
        0.0000515138902046611451
    )
end

function Body.sun()
    return Body.new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0)
end

local function create_bodies()
    local bodies = {
        Body.sun(),
        Body.jupiter(),
        Body.saturn(),
        Body.uranus(),
        Body.neptune()
    }

    local px = 0.0
    local py = 0.0
    local pz = 0.0

    for _, body in ipairs(bodies) do
        px = px + body.vx * body.mass
        py = py + body.vy * body.mass
        pz = pz + body.vz * body.mass
    end

    bodies[1]:offset_momentum(px, py, pz)
    return bodies
end

local NBodySystem = {}

function NBodySystem.new ()
    local obj = { bodies = create_bodies() }
    return setmetatable(obj, {__index = NBodySystem})
end

function NBodySystem:advance (dt)
    for i = 1, #self.bodies do
        local body_i = self.bodies[i]

        for j = i + 1, #self.bodies do
            local body_j = self.bodies[j]
            local dx = body_i.x - body_j.x
            local dy = body_i.y - body_j.y
            local dz = body_i.z - body_j.z

            local dSquared = dx * dx + dy * dy + dz * dz
            local distance = math.sqrt(dSquared)
            local mag = dt / (dSquared * distance)

            body_i.vx = body_i.vx - dx * body_j.mass * mag
            body_i.vy = body_i.vy - dy * body_j.mass * mag
            body_i.vz = body_i.vz - dz * body_j.mass * mag

            body_j.vx = body_j.vx + dx * body_i.mass * mag
            body_j.vy = body_j.vy + dy * body_i.mass * mag
            body_j.vz = body_j.vz + dz * body_i.mass * mag
        end
    end

    for _, body in ipairs(self.bodies) do
        body.x = body.x + dt * body.vx
        body.y = body.y + dt * body.vy
        body.z = body.z + dt * body.vz
    end
end

function NBodySystem:energy ()
    local e = 0.0

    for i = 1, #self.bodies do
        local body_i = self.bodies[i]
        e = e + 0.5 * body_i.mass * (body_i.vx * body_i.vx +
                                     body_i.vy * body_i.vy +
                                     body_i.vz * body_i.vz)

        for j = i + 1, #self.bodies do
            local body_j = self.bodies[j]
            local dx = body_i.x - body_j.x
            local dy = body_i.y - body_j.y
            local dz = body_i.z - body_j.z

            local distance = math.sqrt(dx * dx + dy * dy + dz * dz)
            e = e - (body_i.mass * body_j.mass) / distance;
        end
    end
    return e
end

-- local n = io.read("*n")
local n = 50000

local system = NBodySystem.new()
print(system:energy())
for _ = 1, n do
    system:advance(0.01)
end
print(system:energy())
