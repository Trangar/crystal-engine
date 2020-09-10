# 0.5.0

- Renamed `GuiElementBuilder::with_canvas` to `::canvas`
  - It has a transparent background by default, so the color argument was removed
  - You can update the color by calling `.canvas().with_background_color(...)`
- Added `GuiElement::update_canvas` which allows you to update the element without having to re-create it
  - Note: it only works with canvas elements, texture elements are currently not supported and will panic at runtime
- Added several helper methods to `GuiElementBuilder` to make the new `update_canvas` method more useful.
