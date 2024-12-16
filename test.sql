
.load target/debug/libvsag_sqlite

CREATE VIRTUAL TABLE test_table
USING vsag (dimension=3);

INSERT INTO test_table (id, vec)
    VALUES (1, '[1,2,3]'), (2, '[11,22,33]'), (3, '[111,232,333]');

-- KNN style query
SELECT
    id,
    distance
FROM
    test_table
WHERE
    vec MATCH '[1,2,4]';
