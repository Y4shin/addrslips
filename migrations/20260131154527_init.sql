-- Areas
CREATE TABLE area (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    color INTEGER NOT NULL CHECK (color BETWEEN 0 AND 16777215),
    state INTEGER NOT NULL CHECK (state BETWEEN 0 AND 8),
    image_fname TEXT NOT NULL UNIQUE
);

-- Streets belong to an area
CREATE TABLE street (
    id INTEGER PRIMARY KEY,
    area_id INTEGER NOT NULL,
    name TEXT,
    verified INTEGER NOT NULL DEFAULT 0,
    UNIQUE (area_id, name),
    UNIQUE (area_id, id),
    FOREIGN KEY (area_id) REFERENCES area(id) ON DELETE CASCADE
);

CREATE INDEX idx_street_area_id ON street(area_id);

-- Polyline points: enforce exactly one point per position per street
CREATE TABLE street_polyline_vertices (
    street_id INTEGER NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    x INTEGER NOT NULL CHECK (x BETWEEN 0 AND 4294967295),
    y INTEGER NOT NULL CHECK (y BETWEEN 0 AND 4294967295),
    PRIMARY KEY (street_id, position),
    FOREIGN KEY (street_id) REFERENCES street(id) ON DELETE CASCADE
);

-- Addresses belong to a street (optional), area is derived from street -> area
CREATE TABLE address (
    id INTEGER PRIMARY KEY,
    street_id INTEGER,
    area_id INTEGER NOT NULL,
    house_number TEXT NOT NULL,
    x INTEGER NOT NULL CHECK (x BETWEEN 0 AND 4294967295),
    y INTEGER NOT NULL CHECK (y BETWEEN 0 AND 4294967295),
    circle_radius INTEGER NOT NULL CHECK (circle_radius BETWEEN 0 AND 4294967295),
    confidence REAL NOT NULL,
    verified INTEGER NOT NULL DEFAULT 0,
    estimated_flats INTEGER CHECK (estimated_flats BETWEEN 1 AND 65535),
    UNIQUE (house_number, street_id),
    UNIQUE (area_id, id),
    FOREIGN KEY (area_id) REFERENCES area(id) ON DELETE CASCADE,
    FOREIGN KEY (street_id, area_id) REFERENCES street(id, area_id) ON DELETE SET NULL
);

CREATE INDEX idx_address_street_id ON address(street_id);

-- Teams belong to an area
CREATE TABLE team (
    id INTEGER PRIMARY KEY,
    area_id INTEGER NOT NULL,
    num INTEGER NOT NULL CHECK (num BETWEEN 0 AND 65535),
    UNIQUE (area_id, num),
    UNIQUE (area_id, id),
    FOREIGN KEY (area_id) REFERENCES area(id) ON DELETE CASCADE
);

CREATE INDEX idx_team_area_id ON team(area_id);

-- Team assignments: no duplicated area_id
CREATE TABLE team_assignment (
    team_id INTEGER NOT NULL,
    address_id INTEGER NOT NULL,
    area_id INTEGER NOT NULL,
    UNIQUE (address_id),
    PRIMARY KEY (team_id, address_id),
    FOREIGN KEY (team_id, area_id) REFERENCES team(id, area_id) ON DELETE CASCADE,
    FOREIGN KEY (address_id, area_id) REFERENCES address(id, area_id) ON DELETE CASCADE
);

CREATE INDEX idx_team_assignment_address_id ON team_assignment(address_id);

-- Team bounding polygon vertices
CREATE TABLE team_bounds_vertices (
    team_id INTEGER NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    x INTEGER NOT NULL CHECK (x BETWEEN 0 AND 4294967295),
    y INTEGER NOT NULL CHECK (y BETWEEN 0 AND 4294967295),
    PRIMARY KEY (team_id, position),
    FOREIGN KEY (team_id) REFERENCES team(id) ON DELETE CASCADE
);

-- Project metadata
CREATE TABLE project_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);