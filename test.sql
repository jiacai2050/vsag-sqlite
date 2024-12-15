
.load target/debug/libvsag_sqlite

CREATE VIRTUAL TABLE test_table
USING vsag (dimension=3);

insert into test_table(id, vec) values (1,'[1,2,3]');
insert into test_table(id, vec) values (2,'[11,22,33]');
insert into test_table(id, vec) values (3,'[111,232,333]');

SELECT id,distance FROM test_table where vec match '[1,2,4]';
