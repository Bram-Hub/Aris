use crate::box_chars;
use crate::components::expr_entry::ExprEntry;
use crate::proof_ui_data::ProofUiData;
use crate::util::calculate_lineinfo;
use crate::util::P;

use aris::proofs::pj_to_pjs;
use aris::proofs::Justification;
use aris::proofs::Proof;
use aris::proofs::PJRef;
use aris::proofs::PJSRef;
use aris::rules::Rule;
use aris::rules::RuleClassification;
use aris::rules::RuleM;
use aris::rules::RuleT;

use std::collections::BTreeSet;
use std::fmt;
use std::mem;

use frunk_core::Coprod;
use frunk_core::coproduct::Coproduct;
use strum::IntoEnumIterator;
use yew::prelude::*;

/// Component for editing proofs
pub struct ProofWidget {
    link: ComponentLink<Self>,
    /// The proof being edited with this widget
    prf: P,
    /// UI-specific data associated with the proof, such as intermediate text in
    /// lines that might have parse errors
    pud: ProofUiData<P>,
    /// The currently selected line, highlighted in the UI
    selected_line: Option<PJRef<P>>,
    /// Error message, for if there was an error parsing the proof XML. If this
    /// exists, it is displayed instead of the proof.
    open_error: Option<String>,
    preblob: String,
    props: ProofWidgetProps,
}

#[derive(Debug)]
pub enum LAKItem {
    Line, Subproof
}

#[derive(Debug)]
pub enum LineActionKind {
    Insert { what: LAKItem, after: bool, relative_to: LAKItem, },
    Delete { what: LAKItem },
    SetRule { rule: Rule },
    Select,
    ToggleDependency { dep: Coprod![PJRef<P>, <P as Proof>::SubproofReference] },
}

pub enum ProofWidgetMsg {
    Nop,
    LineChanged(PJRef<P>, String),
    LineAction(LineActionKind, PJRef<P>),
    CallOnProof(Box<dyn FnOnce(&P)>),
}

impl fmt::Debug for ProofWidgetMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ProofWidgetMsg::*;
        match self {
            Nop => f.debug_struct("Nop").finish(),
            LineChanged(r, s) => f.debug_tuple("LineChanged").field(&r).field(&s).finish(),
            LineAction(lak, r) => f.debug_tuple("LineAction").field(&lak).field(&r).finish(),
            CallOnProof(_) => f.debug_struct("CallOnProof").finish(),
        }
    }
}

#[derive(Clone, Properties)]
pub struct ProofWidgetProps {
    pub verbose: bool,
    pub data: Option<Vec<u8>>,
    pub oncreate: Callback<ComponentLink<ProofWidget>>,
}

impl ProofWidget {
    fn render_line_num_dep_checkbox(&self, line: Option<usize>, proofref: Coprod!(PJRef<P>, <P as Proof>::SubproofReference)) -> Html {
        let line = match line {
            Some(line) => line.to_string(),
            None => "".to_string(),
        };
        if let Some(selected_line) = self.selected_line {
            use Coproduct::{Inl, Inr};
            if let Inr(Inl(_)) = selected_line {
                let dep = proofref.clone();
                let selected_line_ = selected_line.clone();
                let toggle_dep = self.link.callback(move |_| {
                    ProofWidgetMsg::LineAction(LineActionKind::ToggleDependency { dep }, selected_line_)
                });
                if self.prf.can_reference_dep(&selected_line, &proofref) {
                    return html! {
                        <button
                            type="button"
                            class="btn btn-secondary"
                            onclick=toggle_dep>

                            { line }
                        </button>
                    };
                }
            }
        }
        html! {
            <button
                type="button"
                class="btn"
                disabled=true>

                { line }
            </button>
        }
    }
    /// Create a drop-down menu allowing the user to select the rule used in a
    /// justification line. This uses the [Bootstrap-submenu][lib] library.
    ///
    /// ## Parameters:
    ///   + `jref` - reference to the justification line containing this menu
    ///   + `cur_rule_name` - name of the current selected rule
    ///
    /// [lib]: https://github.com/vsn4ik/bootstrap-submenu
    fn render_rules_menu(&self, jref: <P as Proof>::JustificationReference, cur_rule_name: &str) -> Html {
        // Create menu items for rule classes
        let menu = RuleClassification::iter()
            .map(|rule_class| {
                // Create menu items for rules in class
                let rules = rule_class
                    .rules()
                    .map(|rule| {
                        let pjref = Coproduct::inject(jref);
                        // Create menu item for rule
                        html! {
                            <button class="dropdown-item" type="button" onclick=self.link.callback(move |_| ProofWidgetMsg::LineAction(LineActionKind::SetRule { rule }, pjref))>
                                { rule.get_name() }
                            </button>
                        }
                    })
                    .collect::<Vec<yew::virtual_dom::VNode>>();
                let rules = yew::virtual_dom::VList::new_with_children(rules, None);
                // Create sub-menu for rule class
                html! {
                    <div class="dropdown dropright dropdown-submenu">
                        <button class="dropdown-item dropdown-toggle" type="button" data-toggle="dropdown"> { rule_class } </button>
                        <div class="dropdown-menu dropdown-scrollbar"> { rules } </div>
                    </div>
                }
            })
            .collect::<Vec<yew::virtual_dom::VNode>>();
        let menu = yew::virtual_dom::VList::new_with_children(menu, None);

        // Create top-level menu button
        html! {
            <div class="dropright">
                <button class="btn btn-primary dropdown-toggle" type="button" data-toggle="dropdown" data-submenu="">
                    { cur_rule_name }
                </button>
                <div class="dropdown-menu">
                    { menu }
                </div>
                <script>
                    { "$('[data-submenu]').submenupicker()" }
                </script>
            </div>
        }
    }
    fn render_justification_widget(&self, jref: <P as Proof>::JustificationReference) -> Html {
        let just = self.prf.lookup_justification_or_die(&jref).expect("proofref should exist in self.prf");

        // Iterator over line dependency badges, for rendering list of
        // dependencies
        let dep_badges = just
            .2
            .iter()
            .map(|dep| {
                let (dep_line, _) = self.pud.ref_to_line_depth[&dep];
                html! {
                    <span class="badge badge-dark m-1"> { dep_line } </span>
                }
            });

        // Iterator over subproof dependency badges, for rendering list of
        // dependencies
        let sdep_badges = just
            .3
            .iter()
            .filter_map(|sdep| self.prf.lookup_subproof(&sdep))
            .map(|sub| {
                let (mut lo, mut hi) = (usize::max_value(), usize::min_value());
                for line in sub.premises().into_iter().map(Coproduct::inject).chain(sub.direct_lines().into_iter().map(Coproduct::inject)) {
                    if let Some((i, _)) = self.pud.ref_to_line_depth.get(&line) {
                        lo = std::cmp::min(lo, *i);
                        hi = std::cmp::max(hi, *i);
                    }
                }
                let sdep_line = format!("{}-{}", lo, hi);
                html! {
                    <span class="badge badge-secondary m-1"> { sdep_line } </span>
                }
            });

        // Node containing all dependency badges, for rendering list of
        // dependencies
        let all_dep_badges = dep_badges.chain(sdep_badges).collect::<Html>();

        let cur_rule_name = just.1.get_name();
        let rule_selector = self.render_rules_menu(jref, &cur_rule_name);
        html! {
            <>
                <td>
                    // Drop-down menu for selecting rules
                    { rule_selector }
                </td>
                <td>
                    // Dependency list
                    <span class="alert alert-secondary small-alert p-1">
                        { all_dep_badges }
                    </span>
                </td>
            </>
        }
    }
    fn render_line_feedback(&self, proofref: PJRef<P>, is_subproof: bool) -> Html {
        use aris::parser::parse;
        let raw_line = match self.pud.ref_to_input.get(&proofref).and_then(|x| if x.len() > 0 { Some(x) } else { None }) {
            None => { return html! { <span></span> }; },
            Some(x) => x,
        };
        match parse(&raw_line).map(|_| self.prf.verify_line(&proofref)) {
            None => html! { <span class="alert alert-warning small-alert">{ "Parse error" }</span> },
            Some(Ok(())) => match proofref {
                Coproduct::Inl(_) => html! {
                    <span class="alert alert-success small-alert">
                        { if is_subproof { "Assumption" } else { "Premise" } }
                    </span>
                },
                _ => html! { <span class="alert small-alert bg-success text-white">{ "Correct" }</span> },
            },
            Some(Err(err)) => {
                html! {
                    <>
                        <button type="button" class="btn btn-danger" data-toggle="popover" data-content=err>
                            { "Error" }
                        </button>
                        <script>
                            { "$('[data-toggle=popover]').popover()" }
                        </script>
                    </>
                }
            },
        }
    }
    fn render_proof_line(&self, line: usize, depth: usize, proofref: PJRef<P>, edge_decoration: &str) -> Html {
        use Coproduct::{Inl, Inr};
        let line_num_dep_checkbox = self.render_line_num_dep_checkbox(Some(line), Coproduct::inject(proofref.clone()));
        let mut indentation = yew::virtual_dom::VList::new();
        for _ in 0..depth {
            //indentation.add_child(html! { <span style="background-color:black">{"-"}</span>});
            //indentation.add_child(html! { <span style="color:white">{"-"}</span>});
            indentation.add_child(html! { <span class="indent"> { box_chars::VERT } </span>});
        }
        indentation.add_child(html! { <span class="indent">{edge_decoration}</span>});
        let proofref_ = proofref.clone();
        let handle_input = self.link.callback(move |value: String| ProofWidgetMsg::LineChanged(proofref_.clone(), value));
        let proofref_ = proofref.clone();
        let select_line = self.link.callback(move |()| ProofWidgetMsg::LineAction(LineActionKind::Select, proofref_.clone()));
        let action_selector = {
            let new_dropdown_item = |text, onclick| {
                html! {
                    <a class="dropdown-item" href="#" onclick=onclick>
                        { text }
                    </a>
                }
            };
            let callback_delete_line = self.link.callback(move |_: web_sys::MouseEvent| {
                ProofWidgetMsg::LineAction(LineActionKind::Delete { what: LAKItem::Line }, proofref_.clone())
            });
            let callback_delete_subproof = self.link.callback(move |_: web_sys::MouseEvent| {
                ProofWidgetMsg::LineAction(LineActionKind::Delete { what: LAKItem::Subproof }, proofref_.clone())
            });
            let callback_insert_line_before_line = self.link.callback(move |_: web_sys::MouseEvent| {
                ProofWidgetMsg::LineAction(LineActionKind::Insert { what: LAKItem::Line, after: false, relative_to: LAKItem::Line }, proofref_.clone())
            });
            let callback_insert_line_after_line = self.link.callback(move |_: web_sys::MouseEvent| {
                ProofWidgetMsg::LineAction(LineActionKind::Insert { what: LAKItem::Line, after: true, relative_to: LAKItem::Line }, proofref_.clone())
            });
            let callback_insert_line_before_subproof = self.link.callback(move |_: web_sys::MouseEvent| {
                ProofWidgetMsg::LineAction(LineActionKind::Insert { what: LAKItem::Line, after: false, relative_to: LAKItem::Subproof }, proofref_.clone())
            });
            let callback_insert_line_after_subproof = self.link.callback(move |_: web_sys::MouseEvent| {
                ProofWidgetMsg::LineAction(LineActionKind::Insert { what: LAKItem::Line, after: true, relative_to: LAKItem::Subproof }, proofref_.clone())
            });
            let callback_insert_subproof_before_line = self.link.callback(move |_: web_sys::MouseEvent| {
                ProofWidgetMsg::LineAction(LineActionKind::Insert { what: LAKItem::Subproof, after: false, relative_to: LAKItem::Line }, proofref_.clone())
            });
            let callback_insert_subproof_after_line = self.link.callback(move |_: web_sys::MouseEvent| {
                ProofWidgetMsg::LineAction(LineActionKind::Insert { what: LAKItem::Subproof, after: true, relative_to: LAKItem::Line }, proofref_.clone())
            });
            let callback_insert_subproof_before_subproof = self.link.callback(move |_: web_sys::MouseEvent| {
                ProofWidgetMsg::LineAction(LineActionKind::Insert { what: LAKItem::Subproof, after: false, relative_to: LAKItem::Subproof }, proofref_.clone())
            });
            let callback_insert_subproof_after_subproof = self.link.callback(move |_: web_sys::MouseEvent| {
                ProofWidgetMsg::LineAction(LineActionKind::Insert { what: LAKItem::Subproof, after: true, relative_to: LAKItem::Subproof }, proofref_.clone())
            });
            let mut options = yew::virtual_dom::VList::new();
            if may_remove_line(&self.prf, &proofref) {
                options.add_child(new_dropdown_item("Delete line", callback_delete_line));
            }
            // Only allow subproof operations on non-root subproofs
            let is_subproof = self.prf.parent_of_line(&pj_to_pjs::<P>(proofref.clone())).is_some();
            if is_subproof {
                options.add_child(new_dropdown_item("Delete subproof", callback_delete_subproof));
                options.add_child(new_dropdown_item("Insert step before this subproof", callback_insert_line_before_subproof));
                options.add_child(new_dropdown_item("Insert step after this subproof", callback_insert_line_after_subproof));
                options.add_child(new_dropdown_item("Insert subproof before this subproof", callback_insert_subproof_before_subproof));
                options.add_child(new_dropdown_item("Insert subproof after this subproof", callback_insert_subproof_after_subproof));
            }
            match proofref {
                Inl(_) => {
                    options.add_child(new_dropdown_item("Insert premise before this premise", callback_insert_line_before_line));
                    options.add_child(new_dropdown_item("Insert premise after this premise", callback_insert_line_after_line));
                },
                Inr(Inl(_)) => {
                    options.add_child(new_dropdown_item("Insert step before this step", callback_insert_line_before_line));
                    options.add_child(new_dropdown_item("Insert step after this step", callback_insert_line_after_line));
                    // Only show subproof creation relative to justification
                    // lines, since it may confuse users to have subproofs
                    // appear after all the premises when they selected a
                    // premise
                    options.add_child(new_dropdown_item("Insert subproof before this step", callback_insert_subproof_before_line));
                    options.add_child(new_dropdown_item("Insert subproof after this step", callback_insert_subproof_after_line));
                },
                Inr(Inr(void)) => match void {},
            }
            html! {
                <div class="dropdown">
                    <button
                        type="button"
                        class="btn btn-secondary dropdown-toggle"
                        id="dropdownMenuButton"
                        data-toggle="dropdown"
                        aria-haspopup="true"
                        aria-expanded="false">

                        { "Action" }
                    </button>
                    <div class="dropdown-menu" aria-labelledby="dropdownMenuButton">
                        { options }
                    </div>
                </div>
            }
        };
        let init_value = self.pud.ref_to_input.get(&proofref).cloned().unwrap_or_default();
        let in_subproof = depth > 0;
        let rule_feedback = self.render_line_feedback(proofref, in_subproof);
        let is_selected_line = self.selected_line == Some(proofref);
        let is_dep_line = match self.selected_line {
            Some(Inr(Inl(selected_line))) => {
                match self.prf.lookup_justification_or_die(&selected_line) {
                    Ok(Justification(_, _, line_deps, _)) => line_deps.contains(&proofref),
                    Err(_) => false,
                }
            }
            _ => false,
        };
        let class = if is_selected_line {
            "proof-line table-info"
        } else if is_dep_line {
            "proof-line table-secondary"
        } else {
            "proof-line"
        };
        let feedback_and_just_widgets = match proofref {
            Inl(_) => {
                // Premise
                html! {
                    <>
                        <td></td>
                        <td> { rule_feedback } </td>
                        <td></td>
                    </>
                }
            }
            Inr(Inl(jref)) => {
                // Justification
                html! {
                    <>
                        <td> { rule_feedback } </td>
                        { self.render_justification_widget(jref) }
                    </>
                }
            }
            Inr(Inr(void)) => match void {},
        };
        html! {
            <tr class=class>
                <td> { line_num_dep_checkbox } </td>
                <td>
                    { indentation }
                    <ExprEntry
                        oninput=handle_input
                        onfocus=select_line
                        init_value=init_value />
                </td>
                { feedback_and_just_widgets }
                <td>{ action_selector }</td>
            </tr>
        }
    }

    fn render_proof(&self, prf: &<P as Proof>::Subproof, sref: Option<<P as Proof>::SubproofReference>, line: &mut usize, depth: &mut usize) -> Html {
        // output has a bool tag to prune subproof spacers with, because VNode's PartialEq doesn't do the right thing
        let mut output: Vec<(Html, bool)> = Vec::new();
        for prem in prf.premises().iter() {
            let edge_decoration = { box_chars::VERT }.to_string();
            output.push((self.render_proof_line(*line, *depth, Coproduct::inject(prem.clone()), &edge_decoration), false));
            *line += 1;
        }
        let dep_checkbox = match sref {
            Some(sr) => self.render_line_num_dep_checkbox(None, Coproduct::inject(sr)),
            None => yew::virtual_dom::VNode::from(yew::virtual_dom::VList::new()),
        };
        let mut spacer = yew::virtual_dom::VList::new();
        spacer.add_child(html! { <td>{ dep_checkbox }</td> });
        //spacer.add_child(html! { <td style="background-color:black"></td> });
        let mut spacer_lines = String::new();
        for _ in 0..*depth {
            spacer_lines.push(box_chars::VERT);
        }
        spacer_lines += &format!("{}{}", box_chars::VERT_RIGHT, box_chars::HORIZ.to_string().repeat(4));
        spacer.add_child(html! { <td> <span class="indent"> {spacer_lines} </span> </td> });

        let spacer = html! { <tr> { spacer } </tr> };

        output.push((spacer, false));
        let prf_lines = prf.lines();
        for (i, lineref) in prf_lines.iter().enumerate() {
            use Coproduct::{Inl, Inr};
            let edge_decoration = if i == prf_lines.len()-1 { box_chars::UP_RIGHT } else { box_chars::VERT }.to_string();
            match lineref {
                Inl(r) => { output.push((self.render_proof_line(*line, *depth, Coproduct::inject(r.clone()), &edge_decoration), false)); *line += 1; },
                Inr(Inl(sr)) => {
                    *depth += 1;
                    //output.push(row_spacer.clone());
                    output.push((self.render_proof(&prf.lookup_subproof(&sr).unwrap(), Some(*sr), line, depth), false));
                    //output.push(row_spacer.clone());
                    *depth -= 1;
                },
                Inr(Inr(void)) => { match *void {} },
            }
        }
        // collapse 2 consecutive row spacers to just 1, formed by adjacent suproofs
        // also remove spacers at the end of an output (since that only occurs if a subproof is the last line of another subproof)
        // This can't be replaced with a range-based loop, since output.len() changes on removal
        {
            let mut i = 0;
            while i < output.len() {
                if output[i].1 && ((i == output.len()-1) || output[i+1].1) {
                    output.remove(i);
                }
                i += 1;
            }
        }
        let output: Vec<Html> = output.into_iter().map(|(x,_)| x).collect();
        let output = yew::virtual_dom::VList::new_with_children(output, None);
        if *depth == 0 {
            html! { <table>{ output }</table> }
        } else {
            yew::virtual_dom::VNode::from(output)
        }
    }
}

fn may_remove_line<P: Proof>(prf: &P, proofref: &PJRef<P>) -> bool {
    use Coproduct::{Inl, Inr};
    let is_premise = match prf.lookup_pj(proofref) {
        Some(Inl(_)) => true,
        Some(Inr(Inl(_))) => false,
        Some(Inr(Inr(void))) => match void {},
        None => panic!("prf.lookup failed in while processing a Delete"),
    };
    let parent = prf.parent_of_line(&pj_to_pjs::<P>(proofref.clone()));
    match parent.and_then(|x| prf.lookup_subproof(&x)) {
        Some(sub) => (is_premise && sub.premises().len() > 1) || (!is_premise && sub.lines().len() > 1),
        None => (is_premise && prf.premises().len() > 1) || (!is_premise && prf.lines().len() > 1)
    }
}

/// Render an alert for an error opening the proof
fn render_open_error(error: &str) -> Html {
    html! {
        <div class="alert alert-danger m-4" role="alert">
            <h4 class="alert-heading"> { "Error opening proof" } </h4>
            <hr />
            <p> { error } </p>
        </div>
    }
}

/// Create a new empty proof, the default proof shown in the UI
fn new_empty_proof() -> P {
    use aris::expression::expression_builders::var;
    let mut proof = P::new();
    proof.add_premise(var(""));
    proof.add_step(Justification(var(""), RuleM::Reit, vec![], vec![]));
    proof
}

impl Component for ProofWidget {
    type Message = ProofWidgetMsg;
    type Properties = ProofWidgetProps;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        props.oncreate.emit(link.clone());
        let (prf, error) = match &props.data {
            Some(data) => {
                let result = aris::proofs::xml_interop::proof_from_xml::<P, _>(&data[..]);
                match result {
                    Ok((prf, _)) => (prf, None),
                    Err(err) => (new_empty_proof(), Some(err)),
                }
            }
            None => (new_empty_proof(), None),
        };

        let pud = ProofUiData::from_proof(&prf);
        let mut tmp = Self {
            link,
            prf,
            pud,
            selected_line: None,
            open_error: error,
            preblob: "".into(),
            props,
        };
        tmp.update(ProofWidgetMsg::Nop);
        tmp
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let mut ret = false;
        if self.props.verbose {
            self.preblob += &format!("{:?}\n", msg);
            ret = true;
        }
        use Coproduct::{Inl, Inr};
        match msg {
            ProofWidgetMsg::Nop => {},
            ProofWidgetMsg::LineChanged(r, input) => {
                self.pud.ref_to_input.insert(r.clone(), input.clone());
                if let Some(e) = aris::parser::parse(&input) {
                    match r {
                        Inl(pr) => { self.prf.with_mut_premise(&pr, |x| { *x = e }); },
                        Inr(Inl(jr)) => { self.prf.with_mut_step(&jr, |x| { x.0 = e }); },
                        Inr(Inr(void)) => match void {},
                    }
                }
                ret = true;
            },
            ProofWidgetMsg::LineAction(LineActionKind::Insert { what, after, relative_to }, orig_ref) => {
                use aris::expression::expression_builders::var;
                let to_select;
                let insertion_point: PJSRef<P> = match relative_to {
                    LAKItem::Line => pj_to_pjs::<P>(orig_ref),
                    LAKItem::Subproof => {
                        let parent = self.prf.parent_of_line(&pj_to_pjs::<P>(orig_ref));
                        match parent {
                            Some(parent) => Coproduct::inject(parent),
                            None => return ret,
                        }
                    }
                };
                match what {
                    LAKItem::Line => match insertion_point {
                        Inl(pr) => {
                            to_select = Inl(self.prf.add_premise_relative(var("__js_ui_blank_premise"), &pr, after));
                        }
                        Inr(Inl(jr)) => {
                            let jsr = Coproduct::inject(jr);
                            to_select = Inr(Inl(self.prf.add_step_relative(Justification(var("__js_ui_blank_step"), RuleM::Reit, vec![], vec![]), &jsr, after)));
                        }
                        Inr(Inr(Inl(sr))) => {
                            let jsr = Coproduct::inject(sr);
                            to_select = Inr(Inl(self.prf.add_step_relative(Justification(var("__js_ui_blank_step"), RuleM::Reit, vec![], vec![]), &jsr, after)));
                        }
                        Inr(Inr(Inr(void))) => match void {},
                    },
                    LAKItem::Subproof => {
                        let sr = self.prf.add_subproof_relative(&insertion_point.subset().unwrap(), after);
                        to_select = self.prf.with_mut_subproof(&sr, |sub| {
                            let to_select = Inl(sub.add_premise(var("__js_ui_blank_premise")));
                            sub.add_step(Justification(var("__js_ui_blank_step"), RuleM::Reit, vec![], vec![]));
                            to_select
                        }).unwrap();
                    },
                }
                self.selected_line = Some(to_select);
                self.preblob += &format!("{:?}\n", self.prf.premises());
                ret = true;
            },
            ProofWidgetMsg::LineAction(LineActionKind::Delete { what }, proofref) => {
                let parent = self.prf.parent_of_line(&pj_to_pjs::<P>(proofref.clone()));
                match what {
                    LAKItem::Line => {
                        fn remove_line_if_allowed<P: Proof, Q: Proof<PremiseReference=<P as Proof>::PremiseReference, JustificationReference=<P as Proof>::JustificationReference>>(prf: &mut Q, pud: &mut ProofUiData<P>, proofref: PJRef<Q>) {
                            if may_remove_line(prf, &proofref) {
                                pud.ref_to_line_depth.remove(&proofref);
                                pud.ref_to_input.remove(&proofref);
                                prf.remove_line(&proofref);
                            }
                        }
                        match parent {
                            Some(sr) => { let pud = &mut self.pud; self.prf.with_mut_subproof(&sr, |sub| { remove_line_if_allowed(sub, pud, proofref); }); },
                            None => { remove_line_if_allowed(&mut self.prf, &mut self.pud, proofref); },
                        }
                    },
                    LAKItem::Subproof => {
                        // TODO: recursively clean out the ProofUiData entries for lines inside a subproof before deletion
                        match parent {
                            Some(sr) => { self.prf.remove_subproof(&sr); },
                            None => {}, // shouldn't delete the root subproof
                        }
                    },
                }
                // Deselect current line to prevent it from pointing to a
                // deleted line. The selected line could be deep inside a
                // deleted subproof, so it's easier to deselect conservatively
                // than to figure out if the selected line is deleted.
                self.selected_line = None;
                ret = true;
            },
            ProofWidgetMsg::LineAction(LineActionKind::SetRule { rule }, proofref) => {
                if let Inr(Inl(jr)) = &proofref {
                    self.prf.with_mut_step(&jr, |j| { j.1 = rule });
                }
                self.selected_line = Some(proofref);
                ret = true;
            },
            ProofWidgetMsg::LineAction(LineActionKind::Select, proofref) => {
                self.selected_line = Some(proofref);
                ret = true;
            },
            ProofWidgetMsg::LineAction(LineActionKind::ToggleDependency { dep }, proofref) => {
                if let Inr(Inl(jr)) = &proofref {
                    self.prf.with_mut_step(&jr, |j| {
                        fn toggle_dep_or_sdep<T: Ord>(dep: T, deps: &mut Vec<T>) {
                            let mut dep_set: BTreeSet<T> = mem::replace(deps, vec![]).into_iter().collect();
                            if dep_set.contains(&dep) {
                                dep_set.remove(&dep);
                            } else {
                                dep_set.insert(dep);
                            }
                            deps.extend(dep_set);
                        }
                        match dep {
                            Inl(lr) => toggle_dep_or_sdep(lr, &mut j.2),
                            Inr(Inl(sr)) => toggle_dep_or_sdep(sr, &mut j.3),
                            Inr(Inr(void)) => match void {},
                        }
                    });
                }
                ret = true;
            },
            ProofWidgetMsg::CallOnProof(f) => {
                f(&self.prf);
            },
        }
        if ret {
            calculate_lineinfo::<P>(&mut self.pud.ref_to_line_depth, self.prf.top_level_proof(), &mut 1, &mut 0);
        }
        ret
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
    fn view(&self) -> Html {
        let widget = match &self.open_error {
            Some(err) => render_open_error(err),
            None => self.render_proof(self.prf.top_level_proof(), None, &mut 1, &mut 0),
        };
        html! {
            <div>
                { widget }
                <div style="display: none">
                    <hr />
                    <pre> { format!("{}\n{:#?}", self.prf, self.prf) } </pre>
                    <hr />
                    <pre> { self.preblob.clone() } </pre>
                </div>
            </div>
        }
    }
}