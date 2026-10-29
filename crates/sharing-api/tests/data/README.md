# Test fixtures

`basic_partitioned/` is the `basic_partitioned` reader test table from
[delta-incubator/dat](https://github.com/delta-incubator/dat) (Apache-2.0),
copied without `.crc` sidecars. It is a two-version table partitioned by
`letter`: version 0 adds files for `a`, `b` and `c`, and version 1 adds `a`,
`e` and `f`.
