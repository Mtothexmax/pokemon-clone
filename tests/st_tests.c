#include "tests/test_framework.h"

// the project's single test composition root: `hz/hzbuild test run --filter st_sim`
void test_app_register(void) {
  extern void app_register_types(void);
  app_register_types();

  REGISTER_TEST_MULTICORE(test_st_sim_walks_the_route);
}
