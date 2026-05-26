## ADDED Requirements

### Requirement: Transição de Dashboard para Login
O sistema SHALL navegar de volta para a tela de Login quando o usuário solicitar logout, limpando o estado do usuário autenticado.

#### Scenario: Logout a partir do Dashboard
- **WHEN** o usuário aciona o logout no Dashboard
- **THEN** o estado de navegação muda para `"login"` e o `AuthUser` é definido como `null`
