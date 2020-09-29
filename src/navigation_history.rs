use crate::{
    buffer::BufferHandle,
    buffer_view::{BufferView, BufferViewCollection},
    client::{ClientCollection, TargetClient},
    cursor::Cursor,
};

pub enum NavigationDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy)]
struct NavigationHistorySnapshot {
    buffer_handle: BufferHandle,
    cursor: Cursor,
}

enum NavigationState {
    IterIndex(usize),
    Insert,
}

pub struct NavigationHistory {
    snapshots: Vec<NavigationHistorySnapshot>,
    state: NavigationState,
}

impl NavigationHistory {
    pub fn save_client_snapshot(
        clients: &mut ClientCollection,
        buffer_views: &BufferViewCollection,
        target_client: TargetClient,
    ) {
        let client = match clients.get_mut(target_client) {
            Some(client) => client,
            None => return,
        };
        let view_handle = match client.current_buffer_view_handle {
            Some(handle) => handle,
            None => return,
        };
        let buffer_view = match buffer_views.get(view_handle) {
            Some(view) => view,
            None => return,
        };

        client.navigation_history.add_snapshot(buffer_view);
    }

    fn add_snapshot(&mut self, buffer_view: &BufferView) {
        let buffer_handle = buffer_view.buffer_handle;
        let cursor = *buffer_view.cursors.main_cursor();

        if let NavigationState::IterIndex(index) = self.state {
            self.snapshots.truncate(index);
        }
        self.state = NavigationState::Insert;

        if let Some(last) = self.snapshots.last() {
            if last.buffer_handle == buffer_handle && last.cursor == cursor {
                return;
            }
        }

        self.snapshots.push(NavigationHistorySnapshot {
            buffer_handle,
            cursor,
        });
    }

    pub fn move_in_history(
        clients: &mut ClientCollection,
        buffer_views: &mut BufferViewCollection,
        target_client: TargetClient,
        direction: NavigationDirection,
    ) {
        let client = match clients.get_mut(target_client) {
            Some(client) => client,
            None => return,
        };

        let history = &mut client.navigation_history;
        let mut history_index = match history.state {
            NavigationState::IterIndex(index) => index,
            NavigationState::Insert => history.snapshots.len(),
        };

        let snapshot = match direction {
            NavigationDirection::Forward => {
                if history_index + 1 >= history.snapshots.len() {
                    return;
                }

                history_index += 1;
                let snapshot = history.snapshots[history_index];
                snapshot
            }
            NavigationDirection::Backward => {
                if history_index == 0 {
                    return;
                }

                if history_index == history.snapshots.len() {
                    if let Some(buffer_view) = client
                        .current_buffer_view_handle
                        .and_then(|h| buffer_views.get(h))
                    {
                        history.add_snapshot(buffer_view)
                    }
                }

                history_index -= 1;
                history.snapshots[history_index]
            }
        };

        history.state = NavigationState::IterIndex(history_index);

        let view_handle = buffer_views
            .buffer_view_handle_from_buffer_handle(target_client, snapshot.buffer_handle);
        client.current_buffer_view_handle = Some(view_handle);

        let mut cursors = match buffer_views.get_mut(view_handle) {
            Some(view) => view.cursors.mut_guard(),
            None => return,
        };
        cursors.clear();
        for cursor in std::slice::from_ref(&snapshot.cursor) {
            cursors.add(*cursor);
        }
    }

    pub fn remove_snapshots_with_buffer_handle(&mut self, buffer_handle: BufferHandle) {
        for i in (0..self.snapshots.len()).rev() {
            if self.snapshots[i].buffer_handle == buffer_handle {
                self.snapshots.remove(i);

                if let NavigationState::IterIndex(index) = &mut self.state {
                    if i <= *index && *index > 0 {
                        *index -= 1;
                    }
                }
            }
        }
    }
}

impl Default for NavigationHistory {
    fn default() -> Self {
        Self {
            snapshots: Vec::default(),
            state: NavigationState::IterIndex(0),
        }
    }
}