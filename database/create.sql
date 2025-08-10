CREATE TABLE users (
    user_id int NOT NULL,
    -- AUTO_INCREMENT,
    user_name varchar(255) NOT NULL,
    email varchar(255) NOT NULL,
    password varchar(255) NOT NULL,
    PRIMARY KEY (user_id)
);

CREATE TABLE clusters (
    cluster_id int NOT NULL AUTO_INCREMENT,
    cluster_name varchar(255) NOT NULL,
    cluster_endpoint varchar(255) NOT NULL,
    user_id int NOT NULL,
    PRIMARY KEY (cluster_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

INSERT INTO users VALUES (0, "alex", "alexbissessur@gmail.com", "password");
