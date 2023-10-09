#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy)]
pub enum InputRole {
    Button0 = 0,
    Pause,
    OneTick,
    Progress1,
    Progress2,
    Progress4,
    Progress8,
    Home,
    Up,
    Right,
    Down,
    Left,
    _EndOfVariants
}

impl InputRole {
    #[allow(non_upper_case_globals)]
    pub const Whole: Self = Self::_EndOfVariants;
}

#[derive(Default, Debug, Clone, Copy)]
pub struct InputRoleState {
    state_and_history: [(bool, u32); InputRole::_EndOfVariants as usize],
}

impl InputRoleState {
    #[inline]
    pub fn clear_all(&mut self) {
        for s_and_h in &mut self.state_and_history {
            *s_and_h = (false, 0);
        }
    }

    pub fn update_history(&mut self) {
        for s_and_h in &mut self.state_and_history {
            s_and_h.1 = (s_and_h.1 << 1) + if s_and_h.0 { 1 } else { 0 };
        }
    }

    #[inline]
    pub fn get(&self, input_role: InputRole) -> (bool, u32) {
        if input_role != InputRole::Whole {
            self.state_and_history[input_role as usize]
        } else {
            let (mut whole_state, mut whole_histroy) = (false, 0);
            for s_and_h in self.state_and_history {
                whole_state |= s_and_h.0;
                whole_histroy |= s_and_h.1;
            }
            (whole_state, whole_histroy)
        }
    }

    #[inline]
    pub fn set(&mut self, input_role: InputRole, state: bool) {
        self.state_and_history[input_role as usize].0 = state;
    }

    #[inline]
    pub fn set_true(&mut self, input_role: InputRole) {
        self.set(input_role, true);
    }

    #[inline]
    pub fn set_false(&mut self, input_role: InputRole) {
        self.set(input_role, false);
    }
}
