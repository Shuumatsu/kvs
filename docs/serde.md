> https://serde.rs/data-model.html

Serde crate 的实现围绕着几个核心的 trait: Serialize, Serializer, Deserialize, Deserializer


> https://docs.serde.rs/serde/trait.Serializer.html

Serializer trait 被用来定义 Serde 数据模型的 serialization half。
Serde 的数据模型提供了 29 种数据类型，我们通过实现 Serializer trait 来给出将这 29 种类型序列化的方法。


>> https://docs.serde.rs/serde/trait.Serialize.html

Serialize trait 被用来定义如何将我们的数据结构转化为 Serde 数据模型的 29 种类型。(serialize 方法并不会返回转化后的数据类型，而是直接在方法种调用 Serializer trait 定义的方法，直接得到序列化后的结果)

Serde 提供了 serde_derive 宏来自动的 for structs and enums 生成 Serialize 实现。



