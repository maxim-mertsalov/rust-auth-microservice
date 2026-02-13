

#[derive(Debug, Clone)]
struct EmailState<'a> {
    original_index: usize,
    local: &'a str,
    domain: &'a str,
    start_reveal: usize,
    end_reveal: usize,
}

impl<'a> EmailState<'a> {
    // Generate the mask only when needed to compare or return
    fn to_masked(&self) -> String {
        let len = self.local.chars().count(); // Count Unicode scalars

        if self.start_reveal + self.end_reveal >= len {
            return format!("{}@{}", self.local, self.domain);
        }

        // Use char_indices to handle UTF-8 safely
        let mut chars = self.local.chars();
        let start: String = chars.by_ref().take(self.start_reveal).collect();

        // The remaining chars in the iterator are the middle + end
        // To get the last N, we collect and slice
        let remaining: Vec<char> = chars.collect();
        let end: String = remaining[remaining.len() - self.end_reveal..].iter().collect();

        format!("{}***{}@{}", start, end, self.domain)
    }

    fn can_reveal_more(&self) -> bool {
        self.start_reveal + self.end_reveal < self.local.chars().count()
    }

    fn increment_reveal(&mut self) {
        if self.start_reveal <= self.end_reveal {
            self.start_reveal += 1;
        } else {
            self.end_reveal += 1;
        }
    }
}

pub fn mask_emails(inputs: &[&String]) -> Vec<String> {
    // Initialize states pointing to the input slices
    let mut states: Vec<EmailState> = inputs
        .iter()
        .enumerate()
        .map(|(i, email)| {
            let mut parts = (*email).splitn(2, '@');
            let local = parts.next().unwrap_or("");
            let domain = parts.next().unwrap_or("");
            EmailState {
                original_index: i,
                local,
                domain,
                start_reveal: 1,
                end_reveal: 2,
            }
        })
        .collect();

    let mut changed = true;
    while changed {
        changed = false;

        // Sort to bring potential collisions together
        states.sort_by(|a, b| a.to_masked().cmp(&b.to_masked()));

        let mut i = 0;
        while i < states.len() {
            let mut j = i + 1;
            let current_mask = states[i].to_masked();

            // Find the range of emails that have the exact same mask
            while j < states.len() && states[j].to_masked() == current_mask {
                j += 1;
            }

            // If j > i + 1, it means we found a group of collisions
            if j > i + 1 {
                for state in &mut states[i..j] {
                    if state.can_reveal_more() {
                        state.increment_reveal();
                        changed = true;
                    }
                }
            }
            i = j;
        }
    }

    // Re-sort to original order and produce final Strings
    states.sort_by_key(|s| s.original_index);
    states.into_iter().map(|s| s.to_masked()).collect()
}