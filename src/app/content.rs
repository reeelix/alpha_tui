use std::collections::HashMap;

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{ListItem, ListState},
};

use crate::runtime::{Runtime, RuntimeArgs};

use super::LIST_ITEM_HIGHLIGHT_COLOR;

/// Used to store the instructions and to remember what instruction should currently be highlighted.
#[derive(Debug, Clone)]
pub struct InstructionListStates {
    instruction_list_state: ListState,
    breakpoint_list_state: ListState,
    instructions: Vec<(usize, String, bool)>, // index, line content, is a breakpoint present
    last_index: i32,
    current_index: i32,
}

#[allow(clippy::cast_sign_loss)]
impl InstructionListStates {
    /// Creates new `InstructionListStates` which hold the current state of the instruction list.
    pub fn new(instructions: &[String], set_breakpoints: Option<&Vec<usize>>) -> Self {
        let mut i = Vec::new();
        for (index, s) in instructions.iter().enumerate() {
            if let Some(v) = set_breakpoints {
                if v.contains(&(index + 1)) {
                    i.push((index, s.clone(), true));
                } else {
                    i.push((index, s.clone(), false));
                }
            } else {
                i.push((index, s.clone(), false));
            }
        }
        InstructionListStates {
            instruction_list_state: ListState::default(),
            breakpoint_list_state: ListState::default(),
            instructions: i,
            last_index: -1,
            current_index: -1,
        }
    }

    /// Returns the instruction states as a vector of list items to be printed in the ui.
    pub fn as_list_items(&self, is_playground: bool) -> Vec<ListItem<'static>> {
        let mut items: Vec<ListItem<'static>> = self
            .instructions()
            .iter()
            .map(|i| {
                let content = vec![Line::from(Span::raw(if is_playground {
                    i.1.clone()
                } else {
                    format!("{:2}: {}", i.0 + 1, i.1)
                }))];
                ListItem::new(content).style(Style::default())
            })
            .collect();
        if is_playground {
            items.reverse();
        }
        items
    }

    /// Selects the line in which the program starts
    pub fn set_start(&mut self, current_instruction_index: i32) {
        self.set(current_instruction_index);
        self.current_index = current_instruction_index - 1;
    }

    /// Used to set the line that should be highlighted
    pub fn set(&mut self, current_instruction_idx: i32) {
        self.current_index = current_instruction_idx - 1_i32;
        if current_instruction_idx - self.last_index == 1 {
            self.instruction_list_state
                .select(Some(current_instruction_idx as usize));
            self.breakpoint_list_state
                .select(Some(current_instruction_idx as usize));
        } else {
            // line jump detected, only increase state by one
            self.instruction_list_state
                .select(Some((self.last_index + 1) as usize));
            self.breakpoint_list_state
                .select(Some((self.last_index + 1) as usize));
        }
        self.last_index = current_instruction_idx - 1;
    }

    /// Used to force the highlight of a specific line.
    ///
    /// Should only be used for visuals if the line is known that should be highlighted.
    pub fn force_set(&mut self, highlight_index: usize) {
        self.instruction_list_state.select(Some(highlight_index));
        self.breakpoint_list_state.select(Some(highlight_index));
    }

    pub fn deselect(&mut self) {
        self.instruction_list_state.select(None);
        self.breakpoint_list_state.select(None);
    }

    /// Updates instruction list and breakpoint list to select the next value
    pub fn set_next_visual(&mut self) {
        list_next(&mut self.instruction_list_state, self.instructions.len());
        list_next(&mut self.breakpoint_list_state, self.instructions.len());
    }

    /// Updates the instructions list and breakpoint list to select the previous value
    pub fn set_prev_visual(&mut self) {
        list_prev(&mut self.instruction_list_state, self.instructions.len());
        list_prev(&mut self.breakpoint_list_state, self.instructions.len());
    }

    pub fn set_instruction_list_state(&mut self, index: Option<usize>) {
        self.instruction_list_state.select(index);
    }

    /// Toggles the breakpoint in the current line
    pub fn toggle_breakpoint(&mut self) {
        let val = self.instructions[self.instruction_list_state.selected().unwrap()].2;
        self.instructions[self.instruction_list_state.selected().unwrap()].2 = !val;
    }

    /// Checks if the current line contains a breakpoint
    pub fn is_breakpoint(&self) -> bool {
        if let Some(idx) = self.instruction_list_state.selected() {
            match self.instructions.get(idx) {
                Some(i) => return i.2,
                None => return false,
            };
        }
        false
        // self.instructions[self.instruction_list_state.selected().unwrap()].2
    }

    /// Returns true if at least one breakpoint is set.
    pub fn breakpoints_set(&self) -> bool {
        for state in &self.instructions {
            if state.2 {
                return true;
            }
        }
        false
    }

    pub fn selected_line(&self) -> Option<usize> {
        self.instruction_list_state.selected()
    }

    pub fn instructions(&self) -> &Vec<(usize, String, bool)> {
        &self.instructions
    }

    pub fn instruction_list_state_mut(&mut self) -> &mut ListState {
        &mut self.instruction_list_state
    }

    pub fn breakpoint_list_state_mut(&mut self) -> &mut ListState {
        &mut self.breakpoint_list_state
    }

    /// Adds a new instruction to the list (display only)
    pub fn add_instruction(&mut self, string: String) {
        self.instructions.push((0, string, false))
    }
}

impl PartialEq for InstructionListStates {
    fn eq(&self, other: &Self) -> bool {
        self.instructions == other.instructions
            && self.last_index == other.last_index
            && self.current_index == other.current_index
    }
}

fn list_next(list_state: &mut ListState, instruction_length: usize) {
    let i = match list_state.selected() {
        Some(i) => {
            if i >= instruction_length - 1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    };
    list_state.select(Some(i));
}

fn list_prev(list_state: &mut ListState, max_index: usize) {
    let i = match list_state.selected() {
        Some(i) => {
            if i == 0 {
                max_index - 1
            } else {
                i - 1
            }
        }
        None => 0,
    };
    list_state.select(Some(i));
}

/// Used to update and set the lists for accumulators, memory cells, stack and call stack.
pub struct MemoryListsManager {
    accumulators: HashMap<usize, (String, bool)>,
    gamma: Option<(Option<i32>, bool)>,
    memory_cells: HashMap<String, (String, bool)>,
    index_memory_cells: HashMap<usize, (String, bool)>,
    stack: Vec<ListItem<'static>>,
    call_stack: Vec<ListItem<'static>>,
}

impl MemoryListsManager {
    /// Creates a new `MemoryListsManager` with the current values of the runtime arguments.
    pub fn new(runtime_args: &RuntimeArgs) -> Self {
        let mut accumulators = HashMap::new();
        for acc in &runtime_args.accumulators {
            accumulators.insert(*acc.0, (format!("{}", acc.1), false));
        }
        //accumulators.sort_by(|a, b| a.0.cmp(&b.0));
        let mut memory_cells = HashMap::new();
        for cell in &runtime_args.memory_cells {
            memory_cells.insert(cell.1.label.clone(), (format!("{}", cell.1), false));
        }
        let mut index_memory_cells = HashMap::new();
        for cell in &runtime_args.index_memory_cells {
            if let Some(v) = cell.1 {
                index_memory_cells.insert(*cell.0, (format!("[{:2}]: {}", *cell.0, *v), false));
            } else {
                index_memory_cells.insert(*cell.0, (format!("[{:2}]: None", *cell.0), false));
            }
        }
        let gamma = runtime_args.gamma.map(|value| (value, false));
        Self {
            accumulators,
            gamma,
            memory_cells,
            index_memory_cells,
            stack: Vec::new(),
            call_stack: Vec::new(),
        }
    }

    /// Updates the lists values.
    /// The old values are compared against the new values, if a value has changed the background color
    /// of that list item is changed.
    ///
    /// `control_flow` is used to update call stack values.
    pub fn update(&mut self, runtime: &Runtime) {
        // Update accumulators
        for acc in &runtime.runtime_args().accumulators {
            let a = match self.accumulators.get_mut(acc.0) {
                Some(value) => value,
                None => {
                    self.accumulators.insert(*acc.0, ("0".to_string(), true));
                    self.accumulators.get_mut(acc.0).unwrap()
                }
            };
            let update = format!("{}", acc.1);
            if update == *a.0 {
                a.1 = false;
            } else {
                *a = (update, true);
            }
        }
        // Update memory_cells
        for cell in &runtime.runtime_args().memory_cells {
            let a = match self.memory_cells.get_mut(&cell.1.label) {
                Some(value) => value,
                None => {
                    self.memory_cells
                        .insert(cell.1.label.clone(), ("0".to_string(), true));
                    self.memory_cells.get_mut(&cell.1.label).unwrap()
                }
            };
            let update = format!("{}", cell.1);
            if update == *a.0 {
                a.1 = false;
            } else {
                *a = (update, true);
            }
        }
        // Update index memory cells
        for cell in &runtime.runtime_args().index_memory_cells {
            if !self.index_memory_cells.contains_key(cell.0) {
                if let Some(v) = cell.1 {
                    self.index_memory_cells
                        .insert(*cell.0, (format!("[{:2}]: {}", cell.0, v), true));
                } else {
                    self.index_memory_cells
                        .insert(*cell.0, (format!("[{:2}]: None", cell.0), true));
                }
                continue;
            }
            let a = self.index_memory_cells.get_mut(cell.0).unwrap();
            let update = match cell.1 {
                Some(v) => format!("[{:2}]: {}", cell.0, v),
                None => format!("[{:2}]: None", cell.0),
            };
            if update == *a.0 {
                a.1 = false;
            } else {
                *a = (update, true);
            }
        }
        // Update gamma
        if let Some(update) = runtime.runtime_args().gamma {
            if let Some(value) = self.gamma.as_mut() {
                if update == value.0 {
                    value.1 = false;
                } else {
                    *value = (update, true);
                }
            } else {
                self.gamma = Some((update, true));
            }
        }
        // Update stack
        let stack_changed = self.stack.len() != runtime.runtime_args().stack.len();
        let mut new_stack: Vec<ListItem<'_>> = runtime
            .runtime_args()
            .stack
            .iter()
            .map(|f| ListItem::new(f.to_string()))
            .collect();
        if stack_changed && !new_stack.is_empty() {
            let last_stack = new_stack
                .pop()
                .unwrap()
                .style(Style::default().bg(LIST_ITEM_HIGHLIGHT_COLOR));
            new_stack.push(last_stack);
        }
        self.stack = new_stack;
        // update call stack
        let call_stack_changed = self.call_stack.len() != runtime.control_flow().call_stack.len();
        let mut new_call_stack: Vec<ListItem<'_>> = runtime
            .control_flow()
            .call_stack
            .iter()
            .map(|f| ListItem::new(format!("{}", f + 1)))
            .collect();
        if call_stack_changed && !new_call_stack.is_empty() {
            let last_stack = new_call_stack
                .pop()
                .unwrap()
                .style(Style::default().bg(LIST_ITEM_HIGHLIGHT_COLOR));
            new_call_stack.push(last_stack);
        }
        self.call_stack = new_call_stack;
    }

    /// Returns the current accumulators as list
    pub fn accumulator_list(&self) -> Vec<ListItem<'static>> {
        let mut list = Vec::new();
        for acc in &self.accumulators {
            let mut item = ListItem::new(acc.1 .0.clone());
            if acc.1 .1 {
                item = item.style(Style::default().bg(LIST_ITEM_HIGHLIGHT_COLOR));
            }
            list.push((item, acc.0));
        }
        list.sort_by(|a, b| a.1.cmp(b.1));
        list.reverse();
        // Insert gamma accumulator if it is in use
        if let Some(value) = self.gamma {
            if let Some(inner_value) = value.0 {
                let mut item = ListItem::new(format!(" γ: {inner_value}"));
                if value.1 {
                    item = item.style(Style::default().bg(LIST_ITEM_HIGHLIGHT_COLOR));
                }
                list.push((item, &0));
            } else {
                let mut item = ListItem::new(" γ: None".to_string());
                if value.1 {
                    item = item.style(Style::default().bg(LIST_ITEM_HIGHLIGHT_COLOR));
                }
                list.push((item, &0));
            }
        }
        list.reverse(); // reverse list to make gamma appear at top of list
        list.iter().map(|f| f.0.clone()).collect()
    }

    /// Returns the current memory cells as list (also contains index memory cells)
    pub fn memory_cell_list(&self) -> Vec<ListItem<'static>> {
        let mut list = Vec::new();
        for cell in &self.memory_cells {
            let mut item = ListItem::new(cell.1 .0.clone());
            if cell.1 .1 {
                item = item.style(Style::default().bg(LIST_ITEM_HIGHLIGHT_COLOR));
            }
            list.push((item, cell.0.clone()));
        }
        list.sort_by(|a, b| a.1.cmp(&b.1));
        // Add index memory cells
        let mut imc: Vec<(usize, bool, String)> = Vec::new();
        for cell in &self.index_memory_cells {
            imc.push((*cell.0, cell.1 .1, cell.1 .0.clone()));
        }
        imc.sort(); // Make sure that index memory cells are properly sorted by index
        for cell in imc {
            let mut item = ListItem::new(cell.2.clone());
            if cell.1 {
                item = item.style(Style::default().bg(LIST_ITEM_HIGHLIGHT_COLOR));
            }
            list.push((item, format!("{}", cell.0)));
        }
        list.iter().map(|f| f.0.clone()).collect()
    }

    /// Returns the stack items as list
    pub fn stack_list(&self) -> Vec<ListItem<'static>> {
        let mut list = self.stack.clone();
        list.reverse();
        list
    }

    /// Returns the call stack items as list
    pub fn call_stack_list(&self) -> Vec<ListItem<'static>> {
        let mut list = self.call_stack.clone();
        list.reverse();
        list
    }
}
