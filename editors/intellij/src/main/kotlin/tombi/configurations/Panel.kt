package tombi.configurations

import com.intellij.openapi.fileChooser.FileChooserDescriptorFactory
import com.intellij.openapi.ui.TextFieldWithBrowseButton
import com.intellij.openapi.ui.emptyText
import com.intellij.ui.components.DialogPanel
import com.intellij.ui.dsl.builder.AlignX
import com.intellij.ui.dsl.builder.Cell
import com.intellij.ui.dsl.builder.Row
import com.intellij.ui.dsl.builder.bindText
import com.intellij.ui.dsl.builder.panel
import com.intellij.ui.dsl.builder.toNonNullableProperty
import tombi.message
import javax.swing.JComponent


/**
 * Create a cell that contain a [TextFieldWithBrowseButton]
 * whose chooser only accepts paths to files.
 */
@Suppress("UnstableApiUsage")
private fun Row.singleFileTextField(): Cell<TextFieldWithBrowseButton> {
    val fileChooserDescriptor = FileChooserDescriptorFactory.singleFile()
    
    return textFieldWithBrowseButton(fileChooserDescriptor, project = null, fileChosen = null)
}


/**
 * Allow this cell to expand to fill the extra horizontal space.
 */
private fun <C : JComponent> Cell<C>.makeFlexible() = apply {
    align(AlignX.FILL)
    resizableColumn()
}


private fun Row.executableInput(block: Cell<TextFieldWithBrowseButton>.() -> Unit) =
    singleFileTextField().makeFlexible().apply(block)


/**
 * Create a [DialogPanel] that is bound to [state].
 * 
 * @see panel
 */
internal fun makePanel(state: TombiConfigurations) = panel {
    row(message("configurations.executable.label")) {
        executableInput {
            bindText(state::executable.toNonNullableProperty("tombi"))
            component.emptyText.text = "tombi"
        }
    }
}
