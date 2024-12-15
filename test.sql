
.load target/debug/libvsag_sqlite

CREATE VIRTUAL TABLE log USING vsag_table(
rows=2,
);

insert into log(id, vec) values (1,'[1,2,3]');
insert into log(vec, id) values ('[11,22,33]',2);

SELECT * FROM log where vec = '[1,2,4]';
-- SELECT score FROM log where id=1 and vec ='[1,2,3]';

-- .schema log;
-- PRAGMA table_info(log);
