CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS file_entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    project_id INTEGER NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

CREATE TABLE if NOT EXISTS user_project (
    user_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
    permission_type TEXT NOT NULL,
    PRIMARY KEY (user_id, project_id),
    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);


CREATE INDEX idx_file_entry_project_id ON file_entry(project_id);
CREATE INDEX idx_user_project_user_id ON user_project(user_id);
CREATE INDEX idx_user_project_project_id ON user_project(project_id);