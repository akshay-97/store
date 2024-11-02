from locust import HttpUser, TaskSet, SequentialTaskSet, task,  between
import uuid
import os

config_set = {
   'version' : 1,
   'base' : uuid.uuid4().hex
}

f1 = open('intent.out', 'w')
f2 = open('attempt.out', 'w')

class PaymentsBehaviour(SequentialTaskSet):
    # payment_id = None
    # attempt_version = [None] * 1
    pm_id = None
    locker_id = None
    fp_id = None
    cust_id = None
    
    def gen(self):
        config_set['version'] += 1
        return (config_set['base'] + str(config_set['version']))
    # @task(1)
    # def intent_create(self):
    #     payment_id = self.gen()
    #     self.payment_id = payment_id
    #     f1.write(self.payment_id)
    #     response = self.client.get('/create/' + self.payment_id + '/kaps', name = "IntentCreate")
    # @task(1)
    # def pay(self):
    #     for i in range(1):
    #         self.attempt_version[i] = self.payment_id + 'version' + str(i)
    #         f2.write(self.attempt_version[i] )
    #         response = self.client.get('/pay/' + self.payment_id + '/' + self.attempt_version[i], name = "AttemptCreate")
    # @task(1)
    # def update_attempt(self):
    #     for i in range(len(self.attempt_version)):
    #         if self.attempt_version[i] is not None:
    #             response = self.client.get('/update_attempt/pay/' + self.attempt_version[i] + '/' + self.payment_id , name = "UpdateAttempt")
    
    # @task(1)
    # def update_intent(self):
    #     response = self.client.get('/update_intent/' + self.payment_id, name = "UpdateIntent")
    
    # @task(1)
    # def retrieve_attempt(self):
    #     response = self.client.get('/retrieve/payment_attempt/' + self.payment_id, name = "RetrieveAllAttempt")
    
    # @task(1)
    # def retrieve_intent(self):
    #     response = self.client.get('/retrieve/payment_intent/' + self.payment_id, name = "RetrieveIntent")

    @task(1)
    def create_payment_method(self):
            self.pm_id = self.gen()
            self.cust_id = uuid.uuid4().hex
            print("customer_id pm_id ", self.cust_id, self.pm_id)
            response = self.client.get('/create/payment_method/' + self.cust_id + '/' + self.pm_id, name = "PmCreate")
    
    @task(1)
    def update_payment_method(self):
        self.locker_id = uuid.uuid4().hex
        self.fp_id = uuid.uuid4().hex
        response = self.client.get('/update/payment_methods/' + self.cust_id + '/' + self.pm_id + '/' + self.fp_id + '/' + self.locker_id, name = "PmUpdate")
    
    @task(1)
    def find_pm_locker_id(self):
        response = self.client.get('/find/payment_method_locker/' + self.locker_id, name =  "PmLockerId")

    @task(1)
    def find_pm_fingerprint(self):
        response = self.client.get('/find/payment_method_fingerprint/' + self.fp_id, name = "PmfingerprintId")
    
    @task(1)
    def find_pm_vust(self):
        response = self.client.get('/find/payment_methods/' + self.cust_id, name = "PmAllCustId")
    
def make():
    return type("MyClass", (PaymentsBehaviour,), {})
    

class WebsiteUser(HttpUser):
    MyClass = make()
    tasks = [MyClass]
