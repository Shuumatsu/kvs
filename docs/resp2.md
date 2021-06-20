> https://redis.io/topics/protocol

RESP (REdis Serialization Protocol) 提供了五种数据类型
- Error，用来传输非二进制安全的错误信息字符串（Error 要求字符串不含换行符）
    - `-{err}\r\n` 其编码在首位用一个 `-` 表示类型是 Error，在末尾用 `\r\n` 表示结束
- Integer，用来传输整型数据
    - `:{number}\r\n` 其编码在首位用一个 `:` 表示类型是 Integer，在末尾用 `\r\n` 表示结束
- Simple String，用来传输非二进制安全的字符串（Simple String 要求字符串不含换行符）
    - `+{str}\r\n` 其编码在首位用一个 `+` 表示类型是 Simple String，在末尾用 `\r\n` 表示结束
- Bluk String，用来传输二级制安全的（大）字符串，字符串最大可以为 512M。
    - `${str.len()}\r\n{str}\r\n` 其编码在首位用一个 `$` 表示类型是 Bluk String，紧接其后是一个数字用于表示字符串的长度，紧接着是一个 `\r\n` 用于表示数字部分结束。然后是给定长度的字符串数据，末尾用 `\r\n` 表示结束。
- Array，用来一次传输多个数据
    - `*{arr.len()}\r\n{...elements}` 其编码在首位用一个 `*` 表示类型是 Array，紧接其后是一个数字用于表示数组的长度，紧接着是一个 `\r\n` 用于表示数字部分结束。接着为各个元素的编码后的数据。

例如我们返回一个包含两个 Bluk String 元素的数组，编码后为 `*2\r\n$2\r\naa\r\n$2\r\nbb\r\n`