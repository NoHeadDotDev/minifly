// Auto-submit forms on checkbox change
document.addEventListener('DOMContentLoaded', function() {
    // Handle todo toggle checkboxes
    const checkboxes = document.querySelectorAll('input[type="checkbox"][onchange]');
    checkboxes.forEach(checkbox => {
        checkbox.addEventListener('change', function() {
            this.form.submit();
        });
    });
    
    // Handle file input labels
    const fileInputs = document.querySelectorAll('.file-input');
    fileInputs.forEach(input => {
        const label = input.nextElementSibling;
        if (label) {
            label.addEventListener('click', function(e) {
                e.preventDefault();
                input.click();
            });
        }
    });
    
    // Add loading state to forms
    const forms = document.querySelectorAll('form');
    forms.forEach(form => {
        form.addEventListener('submit', function() {
            const submitBtn = this.querySelector('button[type="submit"]');
            if (submitBtn) {
                submitBtn.disabled = true;
                submitBtn.textContent = 'Loading...';
            }
        });
    });
    
    // Image preview on upload
    const imageInputs = document.querySelectorAll('input[type="file"][name="image"]');
    imageInputs.forEach(input => {
        input.addEventListener('change', function(e) {
            const file = e.target.files[0];
            if (file && file.type.startsWith('image/')) {
                // Auto-submit the form
                this.form.submit();
            } else if (file) {
                alert('Please select an image file');
                this.value = '';
            }
        });
    });
});