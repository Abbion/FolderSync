function validateIntervalValue(input) {
    let sanitizedValue = input.value.replace(/[^0-9]/g, '').replace(/^0+/, '');
    
    sanitizedValue = sanitizedValue.slice(0, 3);
    input.value = sanitizedValue;
  }
  
  function checkIfValueIsEmpty(input){
    if (input.value.length === 0){
      input.value = "1";
    }
  }