
-- init.sql
-- In phpmyadmin, create a database and name it 'actix_sqlx', then go to SQL tab and paste this code in -> Go
CREATE OR REPLACE USER 'user'@'localhost' IDENTIFIED BY 'password';
CREATE OR REPLACE USER 'user'@'172.17.0.1' IDENTIFIED BY 'password'; -- docker
GRANT ALL PRIVILEGES ON actix_sqlx.* TO 'user'@'localhost' WITH GRANT OPTION;
GRANT ALL PRIVILEGES ON actix_sqlx.* TO 'user'@'172.17.0.1' WITH GRANT OPTION; -- docker
-- refresh permissions:
FLUSH PRIVILEGES;

CREATE TABLE IF NOT EXISTS things(
       id BIGINT UNSIGNED AUTO_INCREMENT NOT NULL PRIMARY KEY,
       -- bool_1 TINYINT(1) NOT NULl,  -- bool
       -- bool_2 BOOLEAN NOT NULL,     -- bool
       i_8 TINYINT NOT NULL,        -- i8
       i_16 SMALLINT NOT NULL,      -- i16
       i_32 INT NOT NULL,           -- i32
       i_64 BIGINT NOT NULL,        -- i64
       -- UNSIGNED                  -- u
       f FLOAT NOT NULL,            -- f32
       f_double DOUBLE NOT NULL,    -- f64
       string VARCHAR(255) NOT NULL -- &str String
       -- CHAR TEXT
);

INSERT INTO things(i_8, i_16, i_32, i_64, f, f_double, string)
VALUES (8, 16, 32, 64, 1.0, 3.14, "hello world");

CREATE TABLE IF NOT EXISTS users(
       -- id INT AUTO_INCREMENT,
       id CHAR(40) PRIMARY KEY,      -- MySql UUID (actually CHAR(36) is enough)
       username VARCHAR(15) NOT NULL,
       email VARCHAR(100) NOT NULL
);

INSERT INTO users(id, username, email)
VALUES (UUID(), 'tunghayho', 'tuanhayho@example.de'),
       (UUID(), 'rustaceans', 'rustaceans@example.de');

