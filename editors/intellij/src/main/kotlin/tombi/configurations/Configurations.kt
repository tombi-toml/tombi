package tombi.configurations


import com.intellij.openapi.components.BaseState
import com.intellij.util.xmlb.XmlSerializerUtil


/**
 * Create a deep copy of [this].
 */
internal fun <S : BaseState> S.copy(): S =
    XmlSerializerUtil.createCopy(this)


/**
 * Settings known to this plugin.
 * 
 * @see TombiConfigurationService
 */
internal class TombiConfigurations : BaseState() {
    /**
     * The Tombi executable to be used in commands.
     * 
     * Defaults to `tombi` if not specified.
     */
    var executable by string("tombi")
}
