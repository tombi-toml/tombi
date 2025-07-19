package tombi.configurations

import com.intellij.openapi.options.Configurable
import com.intellij.openapi.project.ProjectManager
import com.intellij.platform.lsp.api.LspServerManager
import com.intellij.util.xmlb.XmlSerializerUtil
import tombi.server.TombiServerSupportProvider


/**
 * Define a setting panel.
 */
internal class TombiConfigurable : Configurable {
    
    /**
     * The singleton [TombiConfigurationService] instance.
     */
    private val service = TombiConfigurationService.getInstance()
    
    /**
     * The state of configurations that [panel] holds onto.
     * 
     * Once the panel is applied, modifications will be made in-place to this object.
     */
    private val state = service.state.copy()
    
    /**
     * The panel component to be returned by [createComponent].
     * 
     * [isModified], [reset] and [apply] delegate their calls
     * to corresponding methods of this panel.
     * 
     * @see makePanel
     */
    private val panel by lazy { makePanel(state) }
    
    /**
     * The name of the panel.
     * 
     * This function is never called.
     * The name is instead taken from
     * the `configurations.displayName` key
     * in `tombi.properties`.
     */
    override fun getDisplayName() = "Tombi"
    
    /**
     * Create the component that makes up the panel.
     * 
     * @see panel
     */
    override fun createComponent() = panel
    
    /**
     * Whether or not the panel has been modified.
     * If it has, the <i>Apply</i> button will be made available.
     *
     * @see panel
     */
    override fun isModified() = panel.isModified()
    
    /**
     * Revert unsaved modifications to the panel.
     *
     * @see panel
     */
    override fun reset() {
        panel.reset()
    }
    
    /**
     * Save modifications to the global state.
     * 
     * This includes three steps:
     * 
     * * Save modifications to the panel to [state].
     * * Copy the values of the panel's state to the global state.
     * * Restart all servers so that new settings take effect.
     * 
     * @see panel
     */
    override fun apply() {
        panel.apply()
        XmlSerializerUtil.copyBean(state, service.state)
        restartAllServerInstancesAcrossProjects()
    }
    
    private fun restartAllServerInstancesAcrossProjects() {
        val projectManager = ProjectManager.getInstance()
        val openProjects = projectManager.openProjects.filter { !it.isDefault && !it.isDisposed }
        
        openProjects.forEach { project ->
            val serverManager = LspServerManager.getInstance(project)
            
            serverManager.stopAndRestartIfNeeded(TombiServerSupportProvider::class.java)
        }
    }
    
}
