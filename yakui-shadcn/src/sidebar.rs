// Example:
// import { Calendar, Home, Inbox, Search, Settings } from "lucide-react"

// import {
//   Sidebar,
//   SidebarContent,
//   SidebarGroup,
//   SidebarGroupContent,
//   SidebarGroupLabel,
//   SidebarMenu,
//   SidebarMenuButton,
//   SidebarMenuItem,
// } from "@/components/ui/sidebar"

// // Menu items.
// const items = [
//   {
//     title: "Home",
//     url: "#",
//     icon: Home,
//   },
//   {
//     title: "Inbox",
//     url: "#",
//     icon: Inbox,
//   },
//   {
//     title: "Calendar",
//     url: "#",
//     icon: Calendar,
//   },
//   {
//     title: "Search",
//     url: "#",
//     icon: Search,
//   },
//   {
//     title: "Settings",
//     url: "#",
//     icon: Settings,
//   },
// ]

// export function AppSidebar() {
//   return (
//     <Sidebar>
//       <SidebarContent>
//         <SidebarGroup>
//           <SidebarGroupLabel>Application</SidebarGroupLabel>
//           <SidebarGroupContent>
//             <SidebarMenu>
//               {items.map((item) => (
//                 <SidebarMenuItem key={item.title}>
//                   <SidebarMenuButton asChild>
//                     <a href={item.url}>
//                       <item.icon />
//                       <span>{item.title}</span>
//                     </a>
//                   </SidebarMenuButton>
//                 </SidebarMenuItem>
//               ))}
//             </SidebarMenu>
//           </SidebarGroupContent>
//         </SidebarGroup>
//       </SidebarContent>
//     </Sidebar>
//   )
// }
//

use crate::{
    colours::{self, SIDEBAR_BACKGROUND, SIDEBAR_BORDER},
    icons, style, text,
};
use std::f32;
use yakui::{
    Alignment, Constraints, MainAxisAlignment, MainAxisSize, Response, Vec2, colored_box,
    colored_box_container, column, constrained, pad,
    widgets::{Button, ButtonResponse, DynamicButtonStyle, List, Pad},
};

pub enum SidebarItem {
    Group {
        title: String,
        icon: String,
        children: Vec<SidebarItem>,
    },
    Item {
        label: String,
    },
}

pub fn sidebar(title: String, items: &[SidebarItem]) {
    let column = List::column();
    constrained(Constraints::tight(Vec2::new(256.0, f32::MAX)), || {
        colored_box_container(SIDEBAR_BACKGROUND, || {
            pad(Pad::all(8.0), || {
                column.show(|| {
                    header(title.clone());
                    for item in items {
                        draw_item(item);
                    }
                });
            });
        });
    });
}

fn draw_item(item: &SidebarItem) {
    match item {
        SidebarItem::Group {
            title,
            icon,
            children,
        } => {
            pad(Pad::all(8.0), || {
                column(|| {
                    let show = yakui::use_state(|| false);
                    if header_button(icon, title).clicked {
                        show.set(!show.get());
                    }

                    if show.get() {
                        let mut row = List::row();
                        row.main_axis_alignment = MainAxisAlignment::Start;
                        row.main_axis_size = MainAxisSize::Max;
                        row.show(|| {
                            pad(
                                Pad {
                                    left: 16.0,
                                    right: 0.0,
                                    bottom: 0.,
                                    top: 0.,
                                },
                                || {
                                    colored_box(SIDEBAR_BORDER, [1.0, 32.0]);
                                },
                            );
                            column(|| {
                                for child in children {
                                    draw_item(child);
                                }
                            });
                        });
                    }
                });
            });
        }
        SidebarItem::Item { label } => {
            pad(Pad::balanced(8.0, 4.0), || {
                text::body(label.clone());
            });
        }
    }
}

fn header_button(icon: &str, title: &str) -> Response<ButtonResponse> {
    let style = DynamicButtonStyle {
        fill: colours::SIDEBAR_BACKGROUND,
        text: style::text_body(),
        ..Default::default()
    };

    let hover_style = DynamicButtonStyle {
        fill: colours::SIDEBAR_BACKGROUND.adjust(0.9),
        text: style::text_body(),
        ..Default::default()
    };

    let down_style = hover_style.clone();

    Button {
        text: format!("{icon} {title} {}", icons::chevron_down()).into(),
        alignment: Alignment::CENTER,
        padding: Pad::balanced(8.0, 8.0),
        border_radius: 8.0,
        style,
        hover_style,
        down_style,
    }
    .show()
}

fn header(title: String) {
    pad(Pad::all(8.0), || {
        column(|| {
            pad(
                Pad {
                    left: 0.,
                    right: 0.,
                    top: 8.0,
                    bottom: 16.,
                },
                || {
                    text::header(title);
                },
            );
            pad(Pad::vertical(4.0), || {
                text::subheader("Hey there".into());
            });
        });
    });
}
