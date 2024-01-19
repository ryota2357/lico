from math import sqrt

PI = 3.141592653589793
SOLAR_MASS = 4 * PI * PI
DAYS_PER_YER = 365.24

class Body:
    def __init__(self, x, y, z, vx, vy, vz, mass):
        self.x = x
        self.y = y
        self.z = z
        self.vx = vx * DAYS_PER_YER
        self.vy = vy * DAYS_PER_YER
        self.vz = vz * DAYS_PER_YER
        self.mass = mass * SOLAR_MASS

    def offset_momentum(self, px, py, pz):
        self.vx = -(px / SOLAR_MASS)
        self.vy = -(py / SOLAR_MASS)
        self.vz = -(pz / SOLAR_MASS)

    @staticmethod
    def jupiter():
        return Body(
            4.84143144246472090e00,
            -1.16032004402742839e00,
            -1.03622044471123109e-01,
            1.66007664274403694e-03,
            7.69901118419740425e-03,
            -6.90460016972063023e-05,
            9.54791938424326609e-04,
        )

    @staticmethod
    def saturn():
        return Body(
            8.34336671824457987e00,
            4.12479856412430479e00,
            -4.03523417114321381e-01,
            -2.76742510726862411e-03,
            4.99852801234917238e-03,
            2.30417297573763929e-05,
            2.85885980666130812e-04,
        )

    @staticmethod
    def uranus():
        return Body(
            1.28943695621391310e01,
            -1.51111514016986312e01,
            -2.23307578892655734e-01,
            2.96460137564761618e-03,
            2.37847173959480950e-03,
            -2.96589568540237556e-05,
            4.36624404335156298e-05,
        )

    @staticmethod
    def neptune():
        return Body(
            1.53796971148509165e01,
            -2.59193146099879641e01,
            1.79258772950371181e-01,
            2.68067772490389322e-03,
            1.62824170038242295e-03,
            -9.51592254519715870e-05,
            5.15138902046611451e-05,
        )

    @staticmethod
    def sun():
        return Body(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0)

class NBodySystem:
    def __init__(self):
        self._bodies = self._create_bodies()

    @staticmethod
    def _create_bodies():
        bodies = [
            Body.sun(),
            Body.jupiter(),
            Body.saturn(),
            Body.uranus(),
            Body.neptune()
        ]

        px = 0.0
        py = 0.0
        pz = 0.0

        for body in bodies:
            px += body.vx * body.mass
            py += body.vy * body.mass
            pz += body.vz * body.mass

        bodies[0].offset_momentum(px, py, pz)
        return bodies

    def advance(self, dt):
        for i in range(len(self._bodies)):
            body_i = self._bodies[i]

            for j in range(i + 1, len(self._bodies)):
                body_j = self._bodies[j]

                dx = body_i.x - body_j.x
                dy = body_i.y - body_j.y
                dz = body_i.z - body_j.z

                dist2 = dx * dx + dy * dy + dz * dz
                mag = dt / (dist2 * sqrt(dist2))

                body_i.vx -= dx * body_j.mass * mag
                body_i.vy -= dy * body_j.mass * mag
                body_i.vz -= dz * body_j.mass * mag

                body_j.vx += dx * body_i.mass * mag
                body_j.vy += dy * body_i.mass * mag
                body_j.vz += dz * body_i.mass * mag

        for body in self._bodies:
            body.x = body.x + dt * body.vx
            body.y = body.y + dt * body.vy
            body.z = body.z + dt * body.vz

    def energy(self):
        e = 0.0

        for i in range(len(self._bodies)):
            body_i = self._bodies[i]
            e += (
                0.5
                * body_i.mass
                * (
                    body_i.vx * body_i.vx
                    + body_i.vy * body_i.vy
                    + body_i.vz * body_i.vz
                )
            )

            for j in range(i + 1, len(self._bodies)):
                body_j = self._bodies[j]
                dx = body_i.x - body_j.x
                dy = body_i.y - body_j.y
                dz = body_i.z - body_j.z

                distance = sqrt(dx * dx + dy * dy + dz * dz)
                e -= (body_i.mass * body_j.mass) / distance
        return e

def main(n):
    system = NBodySystem()
    print(system.energy())
    for _ in range(n):
        system.advance(0.01)
    print(system.energy())

if __name__ == "__main__":
    # import sys
    # n = int(sys.argv[1])
    n = 50000
    main(n)
