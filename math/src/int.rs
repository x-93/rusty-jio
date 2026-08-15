pub trait SafeInt {
    fn safe_add(self, other: Self) -> Option<Self>
    where
        Self: Sized;
    fn safe_sub(self, other: Self) -> Option<Self>
    where
        Self: Sized;
}

impl SafeInt for i64 {
    fn safe_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    fn safe_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }
}

impl SafeInt for u64 {
    fn safe_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    fn safe_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }
}
