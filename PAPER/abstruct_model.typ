#import "shortcut.typ" : *
#let section(title, body) = [
  #place(
    top,
    float: true,
    scope: "parent",
    clearance: 2em
  )[
    #text(size: 18pt, weight: "bold")[#title] \
    #tab #body
  ]
]
