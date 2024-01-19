const PI = 3.141592653589793;
const SOLAR_MASS = 4 * PI * PI;
const DAYS_PER_YER = 365.24;

class Body {
  constructor(x, y, z, vx, vy, vz, mass) {
    this.x = x;
    this.y = y;
    this.z = z;
    this.vx = vx * DAYS_PER_YER;
    this.vy = vy * DAYS_PER_YER;
    this.vz = vz * DAYS_PER_YER;
    this.mass = mass * SOLAR_MASS;
  }

  offsetMomentum(px, py, pz) {
    this.vx = 0.0 - (px / SOLAR_MASS);
    this.vy = 0.0 - (py / SOLAR_MASS);
    this.vz = 0.0 - (pz / SOLAR_MASS);
  }

  static jupiter() {
    return new Body(
      4.84143144246472090e+00,
      -1.16032004402742839e+00,
      -1.03622044471123109e-01,
      1.66007664274403694e-03,
      7.69901118419740425e-03,
      -6.90460016972063023e-05,
      9.54791938424326609e-04
    );
  }

  static saturn() {
    return new Body(
      8.34336671824457987e+00,
      4.12479856412430479e+00,
      -4.03523417114321381e-01,
      -2.76742510726862411e-03,
      4.99852801234917238e-03,
      2.30417297573763929e-05,
      2.85885980666130812e-04
    );
  }

  static uranus() {
    return new Body(
      1.28943695621391310e+01,
      -1.51111514016986312e+01,
      -2.23307578892655734e-01,
      2.96460137564761618e-03,
      2.37847173959480950e-03,
      -2.96589568540237556e-05,
      4.36624404335156298e-05
    );
  }

  static neptune() {
    return new Body(
      1.53796971148509165e+01,
      -2.59193146099879641e+01,
      1.79258772950371181e-01,
      2.68067772490389322e-03,
      1.62824170038242295e-03,
      -9.51592254519715870e-05,
      5.15138902046611451e-05
    );
  }

  static sun() {
    return new Body(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
  }
}

class NBodySystem {
  constructor() {
    this.bodies = this.#createBodies();
  }

  #createBodies() {
    const bodies = [
      Body.sun(),
      Body.jupiter(),
      Body.saturn(),
      Body.uranus(),
      Body.neptune()
    ];

    let px = 0.0;
    let py = 0.0;
    let pz = 0.0;

    for (const body of bodies) {
      px += body.vx * body.mass;
      py += body.vy * body.mass;
      pz += body.vz * body.mass;
    }

    bodies[0].offsetMomentum(px, py, pz);
    return bodies;
  }

  advance(dt) {
    for (let i = 0; i < this.bodies.length; i++) {
      const bodyI = this.bodies[i];

      for (let j = i + 1; j < this.bodies.length; j++) {
        const bodyJ = this.bodies[j];

        const dx = bodyI.x - bodyJ.x;
        const dy = bodyI.y - bodyJ.y;
        const dz = bodyI.z - bodyJ.z;

        const dist2 = dx * dx + dy * dy + dz * dz;
        const mag = dt / (dist2 * Math.sqrt(dist2));

        bodyI.vx -= dx * bodyJ.mass * mag;
        bodyI.vy -= dy * bodyJ.mass * mag;
        bodyI.vz -= dz * bodyJ.mass * mag;

        bodyJ.vx += dx * bodyI.mass * mag;
        bodyJ.vy += dy * bodyI.mass * mag;
        bodyJ.vz += dz * bodyI.mass * mag;
      }
    }

    for (const body of this.bodies) {
      body.x += dt * body.vx;
      body.y += dt * body.vy;
      body.z += dt * body.vz;
    }
  }

  energy() {
    let e = 0.0;

    for (let i = 0; i < this.bodies.length; i++) {
      const bodyI = this.bodies[i];
      e += 0.5
        * bodyI.mass
        * (
          bodyI.vx * bodyI.vx
          + bodyI.vy * bodyI.vy
          + bodyI.vz * bodyI.vz
        );

      for (let j = i + 1; j < this.bodies.length; j++) {
        const bodyJ = this.bodies[j];
        const dx = bodyI.x - bodyJ.x;
        const dy = bodyI.y - bodyJ.y;
        const dz = bodyI.z - bodyJ.z;

        const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
        e -= (bodyI.mass * bodyJ.mass) / distance;
      }
    }
    return e;
  }
}

const n = 50000

const system = new NBodySystem();
console.log(system.energy())
for (let i = 0; i < n; i++) {
  system.advance(0.01)
}
console.log(system.energy())
