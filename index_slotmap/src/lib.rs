//! 索引分配表。
//! 同时使用SlotMap数据结构实现了一个索引分配表。
//!
//! 索引分配表最初的需求来源于，一个实例可能在多个数据结构中移动， 该实例同时对应一个索引，
//! 在实例移动的前后，索引保持不变，外部总是可以用该索引查找到或删除该实例。
//!
//! 例如：
//! 现有一个以上的数据结构，每个数据结构在插入实例时，
//! 可以返回一个在其自身数据结构中的唯一索引，可以利用该索引查询到插入的实例。
//! 如，现有两个Vec，它们长度都是0，当向Vec1中插入一个实例时，Vec1返回实例的在Vec1中的偏移“0”，
//! 当向Vec2中插入一个实例时，Vec2也返回该实例的在Vec2中的偏移“0”。
//! 现在有另一个数据结构A，其包含这两个数据结构。实例可以在这两个数据结构之间移动，
//! 但实例在插入时，数据结构A返回的索引在实例移动前后不会改变,假如在插入实例时，A返回了一个索引“0”, 
//! 实例移动后，我们希望外部依然可以用“0”查询到该实例。就刚刚的例子，向A中插入一个实例1，指明了初始插入的数据结构时Vec1，
//! 假如我们对外直接但会Vec1返回的实例索引“0”，此时再插入一个实例2，指明了初始插入的数据结构时Vec2，
//! 同理，返回给外部的索引是Vec2返回给我们的索引“0”，当外部用索引“0”到数据结构A中查询实例时，应该返回实例2还是实例1？
//! 如果过一段时间，数据结构A将实例2移动到Vec1中，又如何能够用插入时返回的索引查询到实例2？
//! 这些问题的本质在于：
//! * 数据结构A返回给外部的索引不唯一
//! * 数据结构A将实例从一个数据结构移动到另一个数据结构后，没有保存这种移动变化或状态。移动后，无法确定实例在哪个数据结构中！
//!
//! 本模块为处理这类问题，定义了一个索引分配表，该表负责产生、删除唯一索引、同时为每个索引维护其关心的数据（`class`、`index`）
//! 在上面的例子中，数据结构A负责创建一个索引分配表，当插入实例时，由索引分配表产生一个索引，
//! class用于标记实例在哪个数据结构中（Vec1和Vec2的类型，该类型通常是一个数字，由使用者自行不同值的意义），
//! index用于标记实例在Vec1或Vec2中的偏移，实例移动后，需要修改对应索引中的`class`和`index`.
//!
//! 一个现有的例子是任务池（https://github.com/GaiaWorld/pi_lib/task_pool），在任务池中,
//! 任务可以大致分为队列任务、异步任务、延时任务，这些任务通常放到不同数据结构中。
//! 如延时任务放入到定时轮中，队列任务放入到双端队列中、异步任务放入到权重树中；
//! 当到达规定时间，延时任务需要从定时轮中删除，插入到双端队列或权重树中；
//! 我们希望，外部调用任务池的插入延时任务接口时返回的索引，在移动后依然可用于在任务池中查询或删除对应任务。
//! 
//! 另一个例子更简单的例子，可以参考定时轮（https://github.com/GaiaWorld/pi_lib/local_timer）

extern crate slotmap;

use std::fmt;

use slotmap::{SlotMap, DefaultKey};

/// 索引条目
pub struct IndexEntry<T> {
    /// 索引位置
    pub index: usize,
    /// 索引值
    pub value: T,
}
impl<T: fmt::Debug> fmt::Debug for IndexEntry<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IndexEntry").field(&self.index).field(&self.value).finish()
    }
}
impl<T: Clone> Clone for IndexEntry<T> {
    fn clone(&self) -> Self {
        IndexEntry{
            index: self.index,
            value: self.value.clone(),
        }
    }
}
pub type IndexSlotMap<T> = SlotMap<DefaultKey, IndexEntry<T>>;
