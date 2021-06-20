> https://1drv.ms/b/s!Am4FB3P1c-vg-nAmSuTUrJSb8o6j

一个 Bitcask 实例就是一个 `directory`，并且我们要求任何时候操作系统中只有一个进程在对 Bitcask 进行读写。
在任何时候，`directory` 中有一个 `file` 处于 `active` 状态以供写入。当该 `file` 的大小增长到某个阈值的时候，我们关闭该 `file` 并创建一个新的 `active` 的 `file。`
任何一个 `file` 被（无论出于什么原因）关闭后就将永远的处于 `immutable` 状态。 


我们在写入 `active file` 的时候总是使用 append 的方式，这样我们总是以 sequential 的方式写入，我们就可以避免磁盘寻道。（我们在执行删除操作的时候也并不真的删除，而是写入一个 entry with special tombstone value）
每次写入，一个新的 `entry = [crc, timestamp, key_size, value_sizez, key, value]` 会被 append 到 `active file`。一个 `data file` 就是一串线性排列的 `entry`。

我们在内存中维护了一个 `keydir: HashMap<K, (file_id, value_size, value_pos, timestamp)>` 结构。在每次 append 完成后，我们将用新的数据更新它。
旧的数据依然存在于硬盘上，但是之后的所有读取都将通过更新后的 keydir 读取新数据。每次进行读操作的时候，我们检查 `keydir`，根据 `key` 得到 `(file_id, value_size, value_pos, timestamp)`。

因为我们目前的删除仅仅是通过写入一个新的 `entry` 来标记之前的写入已失效，所以我们终将耗尽所有的空间。我们用 `merge` 操作来解决这个问题。`merge` 操作遍历所有的 `immutable files`，然后输出仅有效的 `data files`。
当 `merge` 完成的后，我们在每个 `merged data file` 旁边创建一个 `fint file`，`fint_file_entry = [timestamp, key_sizie, value_size, value_pos, key]`

当我们打开 Bitcask 时，进程会扫描 `directory` 中所有的 `data files` 来初始化 `keydir`。对于有 `hint file` 的 `data file`，我们选择扫描 `hint file` 来提高初始化速度。