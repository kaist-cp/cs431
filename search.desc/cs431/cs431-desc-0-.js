searchState.loadedDescShard("cs431", 0, "KAIST CS431: Concurrent Programming.\nLocks.\nLock-free data structures.\nCLH lock.\nA type-safe lock.\nA guard that holds the lock and dereferences the inner …\nAn MCS lock.\nAn MCS parking lock.\nRaw lock interface.\nRaw lock interface for the try_lock API.\nA spin lock.\nA ticket lock.\nRaw lock’s token type.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nDestroys the lock and retrieves the lock-protected value.\nAcquires the raw lock.\nAcquires the lock and dereferences the inner value.\nCreates a new lock.\nA sequence lock.\nTries to acquire the raw lock.\nTries to acquire the lock and dereferences the inner value.\nReleases the raw lock.\nA raw sequence lock.\nA reader’s lock guard.\nA sequence lock.\nA writer’s lock guard.\nReleases the reader’s lock.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nDereferences the inner value.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nConsumes this seqlock, returning the underlying data.\nCreates a new raw sequence lock.\nCreates a new sequence lock.\nSafety\nAcquires a reader’s lock.\nSafety\nValidates reads.\nRestarts the read critical section.\nSafety\nTries to upgrade to a writer’s lock.\nValidates reads.\nAcquires a writer’s lock.\nAcquires a writer’s lock.\nReleases a writer’s lock.\nMichael-Scott queue.\nTreiber’s lock-free stack.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns <code>true</code> if the stack is empty.\nLock-free singly linked list.\nCreate a new, empty queue.\nCreates a new, empty stack.\nAttempts to pop the top element from the stack.\nAdds <code>t</code> to the back of the queue.\nPushes a value on top of the stack.\nAttempts to dequeue from the front.\nLinked list cursor.\nSorted singly linked list.\nLinked list node.\nReturns the current node.\nDeletes the current node.\nClean up a chain of logically removed nodes in each …\nDoesn’t preform any cleanup. Gotta go fast. Doesn’t …\nClean up a single logically removed node in each traversal.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nAttempts to delete the value with the Harris strategy.\nLookups the value at <code>key</code> with the Harris-Herlihy-Shavit …\nInsert the value with the Harris strategy.\nLookups the value at <code>key</code> with the Harris strategy.\nDelete the value at <code>key</code> with the Harris-Michael strategy.\nInsert a <code>key</code>-`value`` pair with the Harris-Michael …\nLookups the value at <code>key</code> with the Harris-Michael strategy.\nCreates the head cursor.\nInserts a value between the previous and current node.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nExtracts the inner value.\nLookups the value at the current node.\nCreates a new node.\nCreates a new list.\nCreates a cursor.")